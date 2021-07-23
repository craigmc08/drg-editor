use crate::asset::*;

mod internal;
mod keyboard;
mod operations;
mod plugins;
mod support;
mod tools;

use imgui::*;
use internal::*;
use keyboard::*;
use operations::*;
use plugins::*;
use std::path::Path;
use tinyfiledialogs::{open_file_dialog, save_file_dialog_with_filter};
use tools::*;

const MAIN_WINDOW_FLAGS: WindowFlags = WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS;

pub fn start_editor_with_path(fp: &Path) {
  let editor = match AssetHeader::read_from(fp) {
    Err(err) => Editor {
      state: State::None,
      err: Some(err),
      tool: None,
      keyboard: Keyboard::default(),
    },
    Ok(header) => Editor {
      state: State::Header {
        header,
        path: Box::from(fp.clone()),
        import_editor: ImportEditor::default(),
      },
      err: None,
      tool: None,
      keyboard: Keyboard::default(),
    },
  };
  init_editor(editor)
}

pub fn start_editor_empty() {
  init_editor(Editor::default())
}

pub fn init_editor(editor: Editor) {
  let mut editor = editor;
  let mut operations = Operations::default();

  ToolEditor::register_ops(&mut operations);

  let system = support::init("DRG Editor");

  system.main_loop(move |(width, height), run, ui| {
    let dims = (width as f32, height as f32);
    draw_editor(dims, run, ui, &mut editor, &mut operations);
  })
}

fn draw_editor(
  (width, height): (f32, f32),
  run: &mut bool,
  ui: &Ui,
  editor: &mut Editor,
  ops: &mut Operations,
) {
  let frame_color = ui.push_style_color(StyleColor::WindowBg, [0.1, 0.1, 0.12, 1.0]);

  let menu_height = 35.0;
  let (left, top) = (0.0, menu_height);
  let (width, height) = (width, height - menu_height);

  error_modal(editor, ui);

  // Check keyboard shortcuts
  editor.keyboard.update(ui);
  ops.run(editor, ui);

  draw_menu([0.0, 0.0], [width, menu_height], run, ui, editor);

  if let Some(tool_editor) = &mut editor.tool {
    let mut done = false;
    tool_editor.draw(&mut editor.state, ui, &mut done);
    if done {
      editor.tool = None;
    }
  }

  match &mut editor.state {
    State::None => {
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
    State::Header {
      header,
      import_editor,
      ..
    } => {
      draw_imports_editor(
        [left, top],
        [width / 2.0, height / 2.0],
        ui,
        header,
        import_editor,
      );
      draw_exports_loader(
        [left + width / 2.0, top],
        [width / 2.0, height / 2.0],
        ui,
        editor,
      );
    }
    State::Asset {
      asset,
      import_editor,
      export_editor,
      ..
    } => {
      draw_imports_editor(
        [left, top],
        [width / 2.0, height / 2.0],
        ui,
        &mut asset.header,
        import_editor,
      );
      draw_exports_selector(
        [left + width / 2.0, top],
        [width / 2.0, height / 2.0],
        ui,
        asset,
        export_editor,
      );
      draw_property_selector(
        [left, top + height / 2.0],
        [width / 4.0, height / 2.0],
        ui,
        asset,
        export_editor,
      );
      draw_property_editor(
        [left + width / 4.0, top + height / 2.0],
        [width / 4.0 * 3.0, height / 2.0],
        ui,
        asset,
        export_editor,
      );
    }
  }

  frame_color.pop(ui);
}

fn draw_menu(_pos: [f32; 2], _size: [f32; 2], run: &mut bool, ui: &Ui, editor: &mut Editor) {
  if let Some(main_menu_bar) = ui.begin_main_menu_bar() {
    if let Some(file_menu) = ui.begin_menu(im_str!("File"), true) {
      error_modal(editor, ui);
      // FILE > OPEN
      if MenuItem::new(im_str!("Open"))
        .shortcut(im_str!("Ctrl+O"))
        .build(ui)
      {
        operations::io::open(editor, ui);
      }

      // FILE > SAVE
      if MenuItem::new(im_str!("Save As"))
        .shortcut(im_str!("Ctrl+S"))
        .enabled(editor.state.has_header())
        .build(ui)
      {
        operations::io::save(editor, ui);
      }

      if MenuItem::new(im_str!("Exit")).build(ui) {
        *run = false;
      }

      file_menu.end(ui);
    }

    if let Some(edit_menu) = ui.begin_menu(im_str!("Edit"), true) {
      if let Some(tool_editor) = ToolEditor::menu_items(&editor.state, ui) {
        editor.tool = Some(tool_editor);
      }

      edit_menu.end(ui);
    }

    main_menu_bar.end(ui);
  }
}

fn draw_imports_editor(
  pos: [f32; 2],
  size: [f32; 2],
  ui: &Ui,
  header: &mut AssetHeader,
  editor: &mut ImportEditor,
) {
  let w = Window::new(im_str!("Imports"))
    .flags(MAIN_WINDOW_FLAGS)
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
      for import in header.list_imports() {
        let active = Some(import.name.clone()) == editor.selected_import;

        if ui.radio_button_bool(
          &ImString::from(import.name.to_string(&header.names)),
          active,
        ) {
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
        let import = header
          .list_imports()
          .iter()
          .find(|im| selected == &im.name)
          .expect("Invalid Import select state");

        // TODO: this name deserialization should probably happen after reading
        // imports during Asset::read. But that will require a bunch of work
        let outer = Reference::deserialize(import.outer_index, &header.imports, &header.exports)
          .expect("Invalid Import outer");

        ui.text(format!(
          "Class Package: {}",
          import.class_package.to_string(&header.names)
        ));
        ui.text(format!("Class: {}", import.class.to_string(&header.names)));
        ui.text(format!("Name: {}", import.name.to_string(&header.names)));
        ui.text(format!("Outer: {}", outer.to_string(&header.names)));
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

          input_import(ui, &header, new_import);

          if ui.button(im_str!("Add"), [0.0, 0.0]) {
            header.import(
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

fn draw_exports_loader(pos: [f32; 2], size: [f32; 2], ui: &Ui, editor: &mut Editor) {
  let w = Window::new(im_str!("Exports"))
    .flags(MAIN_WINDOW_FLAGS)
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    error_modal(editor, ui);

    if ui.button(im_str!("Load Export Data [Ctrl+Shift+L]"), [0.0, 0.0]) {
      operations::io::load_exports(editor, ui);
    }
  });
}

fn draw_exports_selector(
  pos: [f32; 2],
  size: [f32; 2],
  ui: &Ui,
  asset: &mut Asset,
  editor: &mut ExportEditor,
) {
  let w = Window::new(im_str!("Exports"))
    .flags(MAIN_WINDOW_FLAGS)
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    for export in asset.list_exports() {
      let active = Some(export.clone()) == editor.selected_export;

      if ui.radio_button_bool(&ImString::from(export.to_string(asset.names())), active) && !active {
        // The selected export is changing, so have to deselect the property
        editor.selected_export = Some(export.clone());
        editor.selected_property = None;
      }
    }
  });
}

fn draw_property_selector(
  pos: [f32; 2],
  size: [f32; 2],
  ui: &Ui,
  asset: &mut Asset,
  editor: &mut ExportEditor,
) {
  let w = Window::new(im_str!("Properties"))
    .flags(MAIN_WINDOW_FLAGS)
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    if let Some(selected) = &editor.selected_export {
      let idx = asset
        .list_exports()
        .iter()
        .position(|x| x == selected)
        .expect("Invalid selected export. Report this crash to the maintainer.");
      let strct = &asset.exports.structs[idx];
      for prop in &strct.properties {
        let active =
          Some(prop.meta.name.clone()) == editor.selected_property.as_ref().map(|s| s.name.clone());
        if ui.radio_button_bool(
          &ImString::from(prop.meta.name.to_string(asset.names())),
          active,
        ) && !active
        {
          editor.selected_property = Some(SelectedProperty {
            name: prop.meta.name.clone(),
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

fn draw_property_editor(
  pos: [f32; 2],
  size: [f32; 2],
  ui: &Ui,
  asset: &mut Asset,
  editor: &mut ExportEditor,
) {
  let w = Window::new(im_str!("Property Editor"))
    .flags(MAIN_WINDOW_FLAGS)
    .resizable(false)
    .collapsible(false)
    .movable(false)
    .position(pos, Condition::Always)
    .size(size, Condition::Always);
  w.build(&ui, || {
    if let Some(selected) = &mut editor.selected_property {
      if selected.dirty {
        ui.text(ImString::from(format!(
          "*{}",
          selected.name.to_string(asset.names())
        )));
      } else {
        ui.text(ImString::from(selected.name.to_string(asset.names())));
      }
      ui.same_line(0.0);
      if ui.button(im_str!("Save"), [0.0, 0.0]) {
        selected.dirty = false;
        selected
          .plugin
          .save(&mut asset.structs_mut()[selected.struct_idx]);
      }

      ui.separator();
      let changed = selected.plugin.input(ui, &asset);
      selected.dirty = selected.dirty || changed;
    } else {
      ui.text("Select a property to edit it");
    }
  });
}

fn input_import(ui: &Ui, header: &AssetHeader, import: &mut EditableImport) {
  ui.input_text(im_str!("Class Package"), &mut import.class_package)
    .resize_buffer(true)
    .build();
  ui.input_text(im_str!("Class"), &mut import.class)
    .resize_buffer(true)
    .build();
  ui.input_text(im_str!("Name"), &mut import.name)
    .resize_buffer(true)
    .build();
  if let Some(new_dep) = input_dependency(ui, "Outer", header, import.outer.clone()) {
    import.outer = new_dep;
  }
}
