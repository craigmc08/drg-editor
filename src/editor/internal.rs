use crate::asset::*;
use crate::editor::plugins::*;
use imgui::*;

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
  pub name: String,
  pub dirty: bool,
  pub struct_idx: usize,
  pub plugin: EditorPlugin,
}

pub struct State {
  pub editor: Option<Editor>,
  pub err: Option<anyhow::Error>,
}

pub struct Editor {
  pub asset: Asset,
  pub new_import: Option<EditableImport>,
  pub selected_import: Option<String>,
  pub selected_export: Option<String>,
  pub selected_property: Option<SelectedProperty>,
}

impl Editor {
  pub fn new(asset: Asset) -> Self {
    Editor {
      asset,
      new_import: None,
      selected_import: None,
      selected_export: None,
      selected_property: None,
    }
  }
}

/// Returns some value if the Dependency is changed
pub fn input_dependency(ui: &Ui, label: &str, dep: Dependency) -> Option<Dependency> {
  let mut new_dep = dep.clone();

  let (prev_item, prev_name) = match dep {
    Dependency::UObject => (0, "".to_string()),
    Dependency::Import(name, _) => (1, name.clone()),
    Dependency::Export(name, _) => (2, name.clone()),
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
      1 => Dependency::import(&prev_name),
      2 => Dependency::export(&prev_name),
      _ => unreachable!(),
    }
  }

  new_dep = match new_dep {
    Dependency::UObject => Dependency::uobject(),
    Dependency::Import(name, _) => {
      let mut new_name = ImString::new(&name);
      new_name.reserve(64);
      ui.input_text(&ImString::new(label), &mut new_name).build();
      changed = changed || new_name != ImString::from(name);
      Dependency::import(new_name.as_ref())
    }
    Dependency::Export(name, _) => {
      let mut new_name = ImString::new(&name);
      ui.input_text(&ImString::new(label), &mut new_name).build();
      changed = changed || new_name != ImString::from(name);
      Dependency::export(new_name.as_ref())
    }
  };

  if changed {
    Some(new_dep)
  } else {
    None
  }
}
