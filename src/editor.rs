use crate::asset::*;

mod support;

use imgui::*;

pub struct NewImport {
  class_package: ImString,
  class: ImString,
  name: ImString,
  outer: Dependency,
}

impl Default for NewImport {
  fn default() -> Self {
    NewImport {
      class_package: ImString::with_capacity(64),
      class: ImString::with_capacity(64),
      name: ImString::with_capacity(64),
      outer: Dependency::uobject(),
    }
  }
}

pub struct Editor {
  asset: Asset,
  new_import: Option<NewImport>,
}

pub fn start_editor(asset: Asset) {
  let mut editor = Editor {
    asset,
    new_import: None,
  };

  let system = support::init("DRG Editor");

  system.main_loop(move |run, ui| {
    draw_editor(run, ui, &mut editor);
  })
}

fn draw_editor(run: &mut bool, ui: &Ui, editor: &mut Editor) {
  let w = Window::new(im_str!("Imports"))
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .size([300.0, 100.0], Condition::FirstUseEver);

  w.build(&ui, || {
    ui.columns(2, im_str!("??"), true);

    for import in editor.asset.list_imports().iter() {
      ui.text(&import.name);
    }

    ui.next_column();
    if ui.button(im_str!("Add Import"), [0.0, 0.0]) {
      editor.new_import = Some(NewImport::default());
      ui.open_popup(im_str!("Add Import"));
    }

    // ADD IMPORT MODAL
    ui.popup_modal(im_str!("Add Import"))
      .always_auto_resize(true)
      .build(|| {
        let new_import = editor
          .new_import
          .as_mut()
          .expect("Add Import modal not initialized properly");

        ui.input_text(im_str!("Class Package"), &mut new_import.class_package)
          .build();
        ui.input_text(im_str!("Class"), &mut new_import.class)
          .build();
        ui.input_text(im_str!("Name"), &mut new_import.name).build();
        new_import.outer = input_dependency(ui, "Outer", new_import.outer.clone());

        if ui.button(im_str!("Add"), [0.0, 0.0]) {
          editor.asset.import(
            new_import.class_package.as_ref(),
            new_import.class.as_ref(),
            new_import.name.as_ref(),
            new_import.outer.clone(),
          );
          editor.new_import = None;
          ui.close_current_popup();
        } else if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
          editor.new_import = None;
          ui.close_current_popup();
        }
      });
  });
}

fn input_dependency(ui: &Ui, label: &str, dep: Dependency) -> Dependency {
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

  if current_item != prev_item {
    new_dep = match current_item {
      0 => Dependency::uobject(),
      1 => Dependency::import(&prev_name),
      2 => Dependency::export(&prev_name),
      _ => unreachable!(),
    }
  }

  match new_dep {
    Dependency::UObject => Dependency::uobject(),
    Dependency::Import(name, _) => {
      let mut new_name = ImString::new(name);
      new_name.reserve(64);
      ui.input_text(&ImString::new(format!("{}", label)), &mut new_name)
        .build();
      Dependency::import(new_name.as_ref())
    }
    Dependency::Export(name, _) => {
      let mut new_name = ImString::new(name);
      ui.input_text(&ImString::new(label), &mut new_name).build();
      Dependency::export(new_name.as_ref())
    }
  }
}
