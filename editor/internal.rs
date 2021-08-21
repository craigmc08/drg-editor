use crate::keyboard::*;
use crate::property_editor::*;
use crate::tools::*;
use drg::asset::property::prop_type::*;
use drg::asset::*;
use imgui::*;
use std::path::Path;

pub struct EditableImport {
  pub class_package: ImString,
  pub class: ImString,
  pub name: ImString,
  pub outer: Reference,
}

impl Default for EditableImport {
  fn default() -> Self {
    EditableImport {
      class_package: ImString::with_capacity(8),
      class: ImString::with_capacity(8),
      name: ImString::with_capacity(8),
      outer: Reference::uobject(),
    }
  }
}

#[derive(Default)]
pub struct ImportEditor {
  pub new_import: Option<EditableImport>,
  pub selected_import: Option<NameVariant>,
}

#[derive(Default)]
pub struct ExportEditor {
  pub selected_export: Option<NameVariant>,
  pub properties_editor: Option<PropertiesEditor>,
}

pub enum State {
  None,
  Header {
    header: AssetHeader,
    path: Box<Path>,
    import_editor: ImportEditor,
  },
  Asset {
    asset: Asset,
    path: Box<Path>,
    import_editor: ImportEditor,
    export_editor: ExportEditor,
  },
}

impl State {
  pub fn is_header(&self) -> bool {
    matches!(self, Self::Header { .. })
  }
  pub fn is_none(&self) -> bool {
    matches!(self, Self::None { .. })
  }

  pub fn has_header(&self) -> bool {
    match self {
      Self::None => false,
      Self::Header { .. } => true,
      Self::Asset { .. } => true,
    }
  }

  /// # Panics
  /// Panics if `!self.has_header()`
  pub fn header(&self) -> &AssetHeader {
    match self {
      Self::None => panic!("editor::internal::State::header on None"),
      Self::Header { header, .. } => header,
      Self::Asset { asset, .. } => &asset.header,
    }
  }

  /// # Panics
  /// Panics if `!self.has_header()`
  pub fn header_mut(&mut self) -> &mut AssetHeader {
    match self {
      Self::None => panic!("editor::internal::State::header on None"),
      Self::Header { header, .. } => header,
      Self::Asset { asset, .. } => &mut asset.header,
    }
  }
}

impl Default for State {
  fn default() -> Self {
    State::None
  }
}

#[derive(Default)]
pub struct Editor {
  pub state: State,
  pub err: Option<anyhow::Error>,
  pub tool: Option<ToolEditor>,
  pub keyboard: Keyboard,
}

pub fn input_prop_type(ui: &Ui, label: &str, value: &mut PropType) -> bool {
  let mut idx = ALL_PROP_TYPES
    .iter()
    .position(|v| v == value)
    .expect("ALL_PROP_TYPES contains all prop types");
  let changed =
    ComboBox::new(&ImString::new(label)).build_simple(ui, &mut idx, &ALL_PROP_TYPES, &|&i| {
      ImString::from(i.to_string()).into()
    });
  if changed {
    *value = ALL_PROP_TYPES[idx];
  };
  changed
}

pub fn input_name_variant(
  ui: &Ui,
  label: &str,
  header: &AssetHeader,
  name: NameVariant,
) -> Option<NameVariant> {
  let NameVariant {
    mut name_idx,
    variant,
  } = name;
  let indices = (0..header.names.names.len()).collect::<Vec<usize>>();

  let name_changed = ComboBox::new(&ImString::new(format!("{} Name", label))).build_simple(
    ui,
    &mut name_idx,
    &indices[..],
    &|&i| ImString::new(&header.names.names[i].name).into(),
  );
  let mut variant_i32 = variant as i32;
  let variant_changed = ui
    .input_int(
      &ImString::new(format!("{} Variant", label)),
      &mut variant_i32,
    )
    .build();

  if name_changed || variant_changed {
    Some(NameVariant {
      name_idx,
      variant: variant_i32 as u32,
    })
  } else {
    None
  }
}

/// Returns some value if the Reference is changed
pub fn input_dependency(
  ui: &Ui,
  label: &str,
  header: &AssetHeader,
  dep: Reference,
) -> Option<Reference> {
  let mut changed = false;

  let mut current_item = match dep {
    Reference::UObject => 0,
    Reference::Import { .. } => 1,
    Reference::Export(_) => 2,
  };
  let mut new_dep: Reference = if ComboBox::new(&ImString::new(format!("{} type", label)))
    .build_simple_string(
      ui,
      &mut current_item,
      &[im_str!("UObject"), im_str!("Import"), im_str!("Export")],
    ) {
    changed = true;
    match current_item {
      0 => Reference::UObject,
      1 => Reference::Import {
        class: header.list_imports()[0].class.clone(),
        name: header.list_imports()[0].name.clone(),
      },
      2 => Reference::Export(header.list_exports()[0].clone()),
      _ => unreachable!(),
    }
  } else {
    dep
  };

  match new_dep.clone() {
    Reference::UObject => {}
    Reference::Import { class, name } => ComboBox::new(im_str!("Import"))
      .preview_value(&ImString::from(format!(
        "{}::{}",
        class.to_string(&header.names),
        name.to_string(&header.names)
      )))
      .build(&ui, || {
        for import in header.list_imports() {
          let is_selected = name == import.name && class == import.class;
          if Selectable::new(&ImString::from(import.to_string(&header.names)))
            .selected(is_selected)
            .build(&ui)
          {
            new_dep = Reference::Import {
              class: import.class.clone(),
              name: import.name.clone(),
            };
            changed = changed || !is_selected;
          }
        }
      }),
    Reference::Export(name) => ComboBox::new(im_str!("Export"))
      .preview_value(&ImString::from(name.to_string(&header.names)))
      .build(&ui, || {
        for export in header.list_exports() {
          let is_selected = name == export;
          if Selectable::new(&ImString::from(export.to_string(&header.names)))
            .selected(is_selected)
            .build(&ui)
          {
            new_dep = Reference::Export(export);
            changed = changed || !is_selected;
          }
        }
      }),
  };

  if changed {
    Some(new_dep)
  } else {
    None
  }
}

pub fn error_modal(editor: &mut Editor, ui: &Ui) {
  ui.popup_modal(im_str!("Error")).build(|| {
    if let Some(err) = &editor.err {
      ui.text(format!("{}", err));
      ui.text("Caused by:");
      err.chain().skip(1).enumerate().for_each(|(i, cause)| {
        ui.text(format!("    {}: {}", i, cause));
      });
      ui.spacing();
      if ui.button(im_str!("Ok"), [0.0, 0.0]) {
        ui.close_current_popup();
      }
    } else {
      ui.close_current_popup();
    }
  });
}
