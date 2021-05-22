use crate::asset::*;
use crate::editor::plugins::*;
use imgui::*;
use std::path::Path;

pub struct EditableImport {
  pub class_package: ImString,
  pub class: ImString,
  pub name: ImString,
  pub outer: Dependency,
}

impl Default for EditableImport {
  fn default() -> Self {
    EditableImport {
      class_package: ImString::with_capacity(64),
      class: ImString::with_capacity(64),
      name: ImString::with_capacity(64),
      outer: Dependency::uobject(),
    }
  }
}

pub struct SelectedProperty {
  pub name: NameVariant,
  pub dirty: bool,
  pub struct_idx: usize,
  pub plugin: EditorPlugin,
}

#[derive(Default)]
pub struct ImportEditor {
  pub new_import: Option<EditableImport>,
  pub selected_import: Option<NameVariant>,
}

#[derive(Default)]
pub struct ExportEditor {
  pub selected_export: Option<NameVariant>,
  pub selected_property: Option<SelectedProperty>,
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

impl Default for State {
  fn default() -> Self {
    State::None
  }
}

pub struct Editor {
  pub state: State,
  pub err: Option<anyhow::Error>,
}

impl Editor {
  pub fn default() -> Self {
    Self {
      state: State::None,
      err: None,
    }
  }
}

/// Returns some value if the Dependency is changed
pub fn input_dependency(
  ui: &Ui,
  label: &str,
  header: &AssetHeader,
  dep: Dependency,
) -> Option<Dependency> {
  let mut new_dep = dep.clone();

  let (prev_item, prev_name) = match dep {
    Dependency::UObject => (0, "".to_string()),
    Dependency::Import(name) => (1, name.to_string()),
    Dependency::Export(name) => (2, name.to_string()),
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
      0 => Dependency::uobject(),
      1 => Dependency::import(prev_name),
      2 => Dependency::export(prev_name),
      _ => unreachable!(),
    }
  }

  match new_dep.clone() {
    Dependency::UObject => {}
    Dependency::Import(name) => ComboBox::new(im_str!("Import"))
      .preview_value(&ImString::from(name.to_string()))
      .build(&ui, || {
        for import in header.list_imports() {
          let is_selected = name == import.name;
          if Selectable::new(&ImString::from(import.name.to_string()))
            .selected(is_selected)
            .build(&ui)
          {
            new_dep = Dependency::Import(import.name.clone());
            changed = changed || !is_selected;
          }
        }
      }),
    Dependency::Export(name) => ComboBox::new(im_str!("Export"))
      .preview_value(&ImString::from(name.to_string()))
      .build(&ui, || {
        for export in header.list_exports() {
          let is_selected = name == export;
          if Selectable::new(&ImString::from(export.to_string()))
            .selected(is_selected)
            .build(&ui)
          {
            new_dep = Dependency::Export(export);
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
