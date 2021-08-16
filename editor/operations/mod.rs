use crate::internal::*;
use crate::keyboard::*;
use imgui::Ui;

pub mod io;
use io::*;

/// Implement this trait if you have operations you want to add to an
/// operations list.
pub trait HasOperations {
  fn register_ops(operations: &mut Operations);
}

pub struct Operations<'a> {
  operations: Vec<Operation<'a>>,
}

impl<'a> Operations<'a> {
  pub fn push(&mut self, op: Operation<'a>) {
    self.operations.push(op);
  }

  pub fn run(&self, editor: &mut Editor, ui: &Ui) {
    for operation in &self.operations {
      if editor.keyboard.chord_available() && operation.is_activated(&editor.keyboard) {
        editor.keyboard.trigger_chord();
        operation.run(editor, ui);
      }
    }
  }
}

impl<'a> Default for Operations<'a> {
  fn default() -> Self {
    Self {
      operations: vec![OPEN, SAVE, LOAD_EXPORTS],
    }
  }
}

type OpFunc = dyn Fn(&mut Editor, &Ui);

pub struct Operation<'a> {
  pub shortcut: Shortcut,
  pub run: &'a OpFunc,
}

#[macro_export]
macro_rules! operation {
  ( $shortcut:expr , $run:expr ) => {
    Operation {
      shortcut: $shortcut,
      run: &$run,
    }
  };
}

impl<'a> Operation<'a> {
  pub fn is_activated(&self, keyboard: &Keyboard) -> bool {
    self.shortcut.is_released(keyboard)
  }

  pub fn run(&self, editor: &mut Editor, ui: &Ui) {
    (self.run)(editor, ui)
  }
}
