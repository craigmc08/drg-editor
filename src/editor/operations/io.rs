use crate::editor::keyboard::*;
use crate::editor::operations::*;
use crate::editor::*;
use crate::operation;
use imgui::*;
use winit::event::VirtualKeyCode;

pub const OPEN: Operation = operation!(Shortcut::new(VirtualKeyCode::O).ctrl(true), open);
pub const SAVE: Operation = operation!(Shortcut::new(VirtualKeyCode::S).ctrl(true), save);
pub const LOAD_EXPORTS: Operation = operation!(
  Shortcut::new(VirtualKeyCode::L).ctrl(true).shift(true),
  load_exports
);

pub fn open(editor: &mut Editor, ui: &Ui) {
  if let Some(fp) = open_file_dialog(
    "Open Asset",
    "",
    Some((&["*.uasset"], "DRG Asset file (*.uasset)")),
  ) {
    match AssetHeader::read_from(fp.as_ref()) {
      Err(err) => {
        editor.err = Some(err);
        ui.open_popup(im_str!("Error"));
      }
      Ok(header) => {
        editor.state = State::Header {
          header,
          path: Box::from(fp.as_ref()),
          import_editor: ImportEditor::default(),
        }
      }
    }
  }
}

pub fn save(editor: &mut Editor, ui: &Ui) {
  if editor.state.is_none() {
    return;
  }

  if let Some(fp) =
    save_file_dialog_with_filter("Save Asset", "", &["*.uasset"], "DRG Asset file (*.uasset)")
  {
    match &mut editor.state {
      State::None => {
        unreachable!()
      }
      State::Header { header, .. } => {
        header.recalculate_offsets();
        if let Err(err) = header.write_out(fp.as_ref()) {
          editor.err = Some(err);
          ui.open_popup(im_str!("Error"));
        }
      }
      State::Asset { asset, .. } => {
        asset.recalculate_offsets();
        if let Err(err) = asset.write_out(fp.as_ref()) {
          editor.err = Some(err);
          ui.open_popup(im_str!("Error"));
        }
      }
    }
  }
}

pub fn load_exports(editor: &mut Editor, ui: &Ui) {
  // Ugly hack to keep state and be able to replace it
  // TODO there is probably a better way to do this
  if editor.state.is_header() {
    if let State::Header {
      header,
      path,
      import_editor,
    } = std::mem::take(&mut editor.state)
    {
      match AssetExports::read_from(&header, &path) {
        Err(err) => {
          editor.err = Some(err);
          editor.state = State::Header {
            header,
            path,
            import_editor,
          };
          ui.open_popup(im_str!("Error"));
        }
        Ok(uexp) => {
          editor.state = State::Asset {
            asset: Asset::new(header, uexp),
            path: path,
            import_editor: import_editor,
            export_editor: ExportEditor::default(),
          }
        }
      }
    }
  }
}
