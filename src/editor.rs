use crate::asset::*;

mod internal;
mod plugins;
mod support;

use imgui::*;
use internal::*;
use plugins::*;
use tinyfiledialogs::{open_file_dialog, save_file_dialog_with_filter};

pub fn start_editor(asset: Option<Asset>) {
  let mut state = State {
    editor: asset.map(Editor::new),
    err: None,
  };

  let system = support::init("DRG Editor");

  system.main_loop(move |(width, height), run, ui| {
    let dims = (width as f32, height as f32);
    draw_editor(dims, run, ui, &mut state);
  })
}

fn draw_editor((width, height): (f32, f32), run: &mut bool, ui: &Ui, state: &mut State) {
  let frame_color = ui.push_style_color(StyleColor::WindowBg, [0.1, 0.1, 0.12, 1.0]);

  let menu_height = 35.0;
  let (left, top) = (0.0, menu_height);
  let (width, height) = (width, height - menu_height);

  draw_menu([0.0, 0.0], [width, menu_height], run, ui, state);

  match &mut state.editor {
    None => {
      let w = Window::new(im_str!("Message"))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .scroll_bar(false)
        .position([left, top], Condition::Always)
        .size([width, height], Condition::Always);
      w.build(&ui, || {
        ui.text("Open a file to start hex modding");
      });
    }
    Some(editor) => {
      draw_imports_editor([left, top], [width / 2.0, height / 2.0], ui, editor);
      draw_exports_selector(
        [left + width / 2.0, top],
        [width / 2.0, height / 2.0],
        ui,
        editor,
      );
      draw_property_selector(
        [left, top + height / 2.0],
        [width / 4.0, height / 2.0],
        ui,
        editor,
      );
      draw_property_editor(
        [left + width / 4.0, top + height / 2.0],
        [width / 4.0 * 3.0, height / 2.0],
        ui,
        editor,
      );
    }
  }

  frame_color.pop(ui);
}

fn draw_menu(pos: [f32; 2], size: [f32; 2], run: &mut bool, ui: &Ui, state: &mut State) {
  let w = Window::new(im_str!("Menu"))
    .title_bar(false)
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .scroll_bar(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    if ui.button(im_str!("Open"), [0.0, 0.0]) {
      if let Some(fp) = open_file_dialog(
        "Open Asset",
        "",
        Some((&["*.uasset"], "DRG Asset file (*.uasset)")),
      ) {
        match Asset::read_from(fp.as_ref()) {
          Err(err) => {
            state.err = Some(err);
            ui.open_popup(im_str!("Error"));
          }
          Ok(asset) => {
            state.editor = Some(Editor::new(asset));
          }
        }
      }
    }

    ui.same_line(0.0);
    if let Some(editor) = &mut state.editor {
      // Only show the Save button if a file is opened
      if ui.button(im_str!("Save As"), [0.0, 0.0]) {
        if let Some(fp) =
          save_file_dialog_with_filter("Save Asset", "", &["*.uasset"], "DRG Asset file (*.uasset)")
        {
          editor.asset.recalculate_offsets();
          match editor.asset.write_out(fp.as_ref()) {
            Err(err) => {
              state.err = Some(err);
              ui.open_popup(im_str!("Error"));
            }
            Ok(_) => {}
          }
        }
      }
      ui.same_line(0.0);
    }

    ui.popup_modal(im_str!("Error")).build(|| {
      if let Some(err) = &state.err {
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

    if ui.button(im_str!("Quit"), [0.0, 0.0]) {
      *run = false;
    }
  });
}

fn draw_imports_editor(pos: [f32; 2], size: [f32; 2], ui: &Ui, editor: &mut Editor) {
  let w = Window::new(im_str!("Imports"))
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .scroll_bar(false)
    .size(size, Condition::Always);

  w.build(&ui, || {
    ui.columns(2, im_str!("??"), true);

    // LIST PANE
    let list_w = ChildWindow::new("importlist")
      .horizontal_scrollbar(true)
      .movable(false);
    list_w.build(&ui, || {
      for import in editor.asset.list_imports() {
        let active = Some(import.name.clone()) == editor.selected_import;

        if ui.radio_button_bool(&ImString::from(import.name.to_string()), active) {
          editor.selected_import = Some(import.name.clone());
        }
      }
    });

    // EDITING PANE
    ui.next_column();
    let edit_w = ChildWindow::new("importedit")
      .movable(false)
      .horizontal_scrollbar(true);
    edit_w.build(&ui, || {
      if ui.button(im_str!("Add Import"), [0.0, 0.0]) {
        editor.new_import = Some(EditableImport::default());
        ui.open_popup(im_str!("Add Import"));
      }
      ui.separator();
      if let Some(selected) = &editor.selected_import {
        let import = &editor
          .asset
          .imports
          .objects
          .iter()
          .find(|im| selected == &im.name)
          .expect("Invalid Import select state");

        ui.text(format!("Class Package: {}", import.class_package));
        ui.text(format!("Class: {}", import.class_package));
        ui.text(format!("Name: {}", import.name));
      } else {
        ui.text("Select an import");
      }

      // ADD IMPORT MODAL
      ui.popup_modal(im_str!("Add Import"))
        .always_auto_resize(true)
        .build(|| {
          let new_import = editor
            .new_import
            .as_mut()
            .expect("Add Import modal not initialized properly");

          input_import(ui, new_import);

          if ui.button(im_str!("Add"), [0.0, 0.0]) {
            editor.asset.import(
              new_import.class_package.as_ref(),
              new_import.class.as_ref(),
              new_import.name.as_ref(),
              new_import.outer.clone(),
            );
            editor.new_import = None;
            ui.close_current_popup();
          }
          ui.same_line(0.0);
          if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
            editor.new_import = None;
            ui.close_current_popup();
          }
        });
    });
  });
}

fn draw_exports_selector(pos: [f32; 2], size: [f32; 2], ui: &Ui, editor: &mut Editor) {
  let w = Window::new(im_str!("Exports"))
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    for export in editor.asset.list_exports() {
      let active = Some(export.clone()) == editor.selected_export;

      if ui.radio_button_bool(&ImString::from(export.to_string()), active) {
        if !active {
          // The selected export is changing, so have to deselect the property
          editor.selected_export = Some(export.clone());
          editor.selected_property = None;
        }
      }
    }
  });
}

fn draw_property_selector(pos: [f32; 2], size: [f32; 2], ui: &Ui, editor: &mut Editor) {
  let w = Window::new(im_str!("Properties"))
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    if let Some(selected) = &editor.selected_export {
      let idx = editor
        .asset
        .exports
        .exports
        .iter()
        .position(|x| &x.object_name == selected)
        .expect("Invalid selected export. Report this crash to the maintainer.");
      let strct = &editor.asset.structs[idx];
      for prop in &strct.properties {
        let active =
          Some(prop.name.clone()) == editor.selected_property.as_ref().map(|s| s.name.clone());
        if ui.radio_button_bool(&ImString::from(prop.name.to_string()), active) && !active {
          editor.selected_property = Some(SelectedProperty {
            name: prop.name.clone(),
            dirty: false,
            plugin: EditorPlugin::new(prop),
            struct_idx: idx,
          });
        }
      }
    } else {
      ui.text("Select an export");
    }
  });
}

fn draw_property_editor(pos: [f32; 2], size: [f32; 2], ui: &Ui, editor: &mut Editor) {
  let w = Window::new(im_str!("Property Editor"))
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    if let Some(selected) = &mut editor.selected_property {
      if selected.dirty {
        ui.text(ImString::from(format!("*{}", selected.name)));
      } else {
        ui.text(ImString::from(selected.name.to_string()));
      }
      ui.same_line(0.0);
      if ui.button(im_str!("Save"), [0.0, 0.0]) {
        selected.dirty = false;
        selected
          .plugin
          .save(&mut editor.asset.structs[selected.struct_idx]);
      }

      ui.separator();
      let changed = selected.plugin.input(ui, &editor.asset);
      selected.dirty = selected.dirty || changed;
    } else {
      ui.text("Select a property to edit it");
    }
  });
}

fn input_import(ui: &Ui, import: &mut EditableImport) {
  ui.input_text(im_str!("Class Package"), &mut import.class_package)
    .build();
  ui.input_text(im_str!("Class"), &mut import.class).build();
  ui.input_text(im_str!("Name"), &mut import.name).build();
  if let Some(new_dep) = input_dependency(ui, "Outer", import.outer.clone()) {
    import.outer = new_dep;
  }
}
