use crate::editor::internal::*;
use crate::editor::keyboard::*;
use imgui::Ui;

pub mod io;
use io::*;

pub const OPERATIONS: &[Operation] = &[OPEN, SAVE, LOAD_EXPORTS];

type OpFunc = dyn Fn(&mut Editor, &Ui);

pub struct Operation<'a> {
  shortcut: Shortcut,
  run: &'a OpFunc,
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
