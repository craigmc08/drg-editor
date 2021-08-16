use crate::keyboard::*;
use crate::property_editor::*;
use crate::tools::*;
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

/// Returns some value if the Reference is changed
pub fn input_dependency(
  ui: &Ui,
  label: &str,
  header: &AssetHeader,
  dep: Reference,
) -> Option<Reference> {
  let mut new_dep = dep.clone();

  let (prev_item, prev_name) = match dep {
    Reference::UObject => (0, None),
    Reference::Import(name) => (1, Some(name)),
    Reference::Export(name) => (2, Some(name)),
  };

  let mut current_item = prev_item;
  ComboBox::new(&ImString::new(format!("{} type", label))).build_simple_string(
    ui,
    &mut current_item,
    &[im_str!("UObject"), im_str!("Import"), im_str!("Export")],
  );

  let mut changed = false;
  if current_item != prev_item {
    changed = true;
    new_dep = match current_item {
      0 => Reference::uobject(),
      1 => Reference::import(prev_name.unwrap_or_else(|| header.list_imports()[0].name.clone())),
      2 => Reference::export(prev_name.unwrap_or_else(|| header.list_exports()[0].clone())),
      _ => unreachable!(),
    }
  }

  match new_dep.clone() {
    Reference::UObject => {}
    Reference::Import(name) => ComboBox::new(im_str!("Import"))
      .preview_value(&ImString::from(name.to_string(&header.names)))
      .build(&ui, || {
        for import in header.list_imports() {
          let is_selected = name == import.name;
          if Selectable::new(&ImString::from(import.name.to_string(&header.names)))
            .selected(is_selected)
            .build(&ui)
          {
            new_dep = Reference::Import(import.name.clone());
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
