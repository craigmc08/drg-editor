use crate::editor::internal::*;
use crate::editor::keyboard::*;
use crate::editor::operations::*;
use crate::operation;
use drg::asset::*;
use imgui::*;
use winit::event::VirtualKeyCode;

pub enum ToolEditor {
  NameReplacer(NameReplacerTool),
}

macro_rules! tool {
  ($v:ident) => {
    (match $v {
      ToolEditor::NameReplacer(tool) => tool,
    })
  };
}

impl ToolEditor {
  /// Draws the menu items and returns a new ToolEditor if one was clicked
  ///
  /// # Example
  ///
  /// ```
  /// if let Some(edit_menu) = ui.begin_menu("Edit", true) {
  ///   if let Some(tool_editor) = ToolEditor::menu_items(editor, ui) {
  ///     editor.tool = Some(tool_editor)
  ///   }
  /// }
  /// ```
  pub fn menu_items(state: &State, ui: &Ui) -> Option<Self> {
    if MenuItem::new(im_str!("Replace Name"))
      .shortcut(im_str!("Ctrl+R"))
      .enabled(state.has_header())
      .build(ui)
    {
      return Some(ToolEditor::NameReplacer(NameReplacerTool::default()));
    }

    None
  }

  pub fn draw(&mut self, state: &mut State, ui: &Ui, done: &mut bool) {
    let w = Window::new(tool!(self).tool_name()).always_auto_resize(true);
    w.build(&ui, || {
      tool!(self).draw(&state, ui);
      if ui.button(tool!(self).finish_text(), [0.0, 0.0]) {
        tool!(self).finish(state);
        *done = true;
      }
      ui.same_line(0.0);
      if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
        *done = true;
      }
    });
  }

  fn start_name_replacer(editor: &mut Editor, _ui: &Ui) {
    editor.tool = Some(Self::NameReplacer(NameReplacerTool::default()));
  }
}

impl HasOperations for ToolEditor {
  fn register_ops(ops: &mut Operations) {
    ops.push(operation!(
      Shortcut::new(VirtualKeyCode::R).ctrl(true),
      ToolEditor::start_name_replacer
    ));
  }
}

trait Tool {
  fn tool_name(&self) -> &'static ImStr;
  fn finish_text(&self) -> &'static ImStr;

  fn draw(&mut self, state: &State, ui: &Ui);
  fn finish(&self, state: &mut State);
}

pub struct NameReplacerTool {
  from: String,
  to: ImString,
}

impl Tool for NameReplacerTool {
  fn tool_name(&self) -> &'static ImStr {
    im_str!("Replace Name")
  }
  fn finish_text(&self) -> &'static ImStr {
    im_str!("Replace")
  }
  fn draw(&mut self, state: &State, ui: &Ui) {
    if !state.has_header() {
      panic!("NameReplacerTool created without header in state");
    }

    ComboBox::new(im_str!("Original"))
      .preview_value(&ImString::from(self.from.to_string()))
      .build(&ui, || {
        for name in state.header().names.names.iter().map(|name| &name.name) {
          let is_selected = &self.from == name;
          if Selectable::new(&ImString::from(name.to_string()))
            .selected(is_selected)
            .build(ui)
          {
            self.from = name.clone();
          }
        }
      });
    ui.input_text(im_str!("Replace With"), &mut self.to)
      .resize_buffer(true)
      .build();
  }
  fn finish(&self, state: &mut State) {
    if state.has_header() {
      let header = state.header_mut();
      let from = NameVariant::parse(&self.from, &header.names);
      header.names.names[from.name_idx].name = self.to.to_string();
    }
  }
}

impl Default for NameReplacerTool {
  fn default() -> Self {
    NameReplacerTool {
      from: "".to_string(),
      to: ImString::with_capacity(8),
    }
  }
}
