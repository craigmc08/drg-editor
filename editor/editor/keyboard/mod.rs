use imgui::Ui;
use std::time::{Duration, SystemTime};
use winit::event::VirtualKeyCode;

pub const CHORD_COOLDOWN: Duration = Duration::from_millis(500);

#[derive(Clone, Copy)]
pub struct Shortcut {
  ctrl: bool,
  shift: bool,
  alt: bool,
  key: VirtualKeyCode,
}

impl Shortcut {
  pub const fn new(key: VirtualKeyCode) -> Self {
    Self {
      ctrl: false,
      shift: false,
      alt: false,
      key,
    }
  }

  pub const fn ctrl(self, required: bool) -> Self {
    Self {
      ctrl: required,
      ..self
    }
  }

  pub const fn shift(self, required: bool) -> Self {
    Self {
      shift: required,
      ..self
    }
  }

  pub const fn alt(self, required: bool) -> Self {
    Self {
      alt: required,
      ..self
    }
  }

  /// Returns true if every key in the chord was just released or is pressed
  pub fn is_released(&self, keys: &Keyboard) -> bool {
    if self.ctrl != keys.is_ctrl_down() {
      return false;
    }
    if self.shift != keys.is_shift_down() {
      return false;
    }
    if self.alt != keys.is_alt_down() {
      return false;
    }
    if !keys.is_key_down(self.key) {
      return false;
    }
    true
  }
}

pub struct Keyboard {
  keys_down_last: [bool; 512],
  keys_down: [bool; 512],
  chord_last_time: SystemTime,
}

impl Keyboard {
  pub fn update(&mut self, ui: &Ui) {
    for (i, key) in ui.io().keys_down.iter().enumerate() {
      self.keys_down_last[i] = self.keys_down[i];
      self.keys_down[i] = *key;
    }
  }

  pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
    self.keys_down[key as usize]
  }

  pub fn is_key_up(&self, key: VirtualKeyCode) -> bool {
    !self.is_key_down(key)
  }

  pub fn is_key_released(&self, key: VirtualKeyCode) -> bool {
    let key = key as usize;
    self.keys_down_last[key] && !self.keys_down[key]
  }

  /// Returns true if the key is down or was down
  pub fn is_key_active(&self, key: VirtualKeyCode) -> bool {
    let key = key as usize;
    self.keys_down_last[key] || self.keys_down[key]
  }

  pub fn is_ctrl_down(&self) -> bool {
    self.keys_down[VirtualKeyCode::LControl as usize]
      || self.keys_down[VirtualKeyCode::RControl as usize]
  }
  pub fn is_shift_down(&self) -> bool {
    self.keys_down[VirtualKeyCode::LShift as usize]
      || self.keys_down[VirtualKeyCode::RControl as usize]
  }
  pub fn is_alt_down(&self) -> bool {
    self.keys_down[VirtualKeyCode::LAlt as usize] || self.keys_down[VirtualKeyCode::RAlt as usize]
  }

  pub fn chord_available(&self) -> bool {
    CHORD_COOLDOWN < self.chord_last_time.elapsed().expect("SystemTime error???")
  }
  pub fn trigger_chord(&mut self) {
    self.chord_last_time = SystemTime::now()
  }
}

impl Default for Keyboard {
  fn default() -> Self {
    Self {
      keys_down_last: [false; 512],
      keys_down: [false; 512],
      chord_last_time: SystemTime::now(),
    }
  }
}
