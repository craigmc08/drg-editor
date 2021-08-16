use crate::plugins::*;
use drg::bindings::AsProperty;
use drg::*;
use imgui::*;

pub struct PropertiesEditor {
  selected: Option<SelectedProperty>,
  window_flags: WindowFlags,
}

struct SelectedProperty {
  pub name: NameVariant,
  pub dirty: bool,
  pub plugin: EditorPlugin,
}

impl PropertiesEditor {
  pub fn draw(
    &mut self,
    (left, top): (f32, f32),
    (width, height): (f32, f32),
    ui: &Ui,
    header: &AssetHeader,
    properties: &mut Vec<Property>,
  ) {
    self.draw_property_selector([left, top], [width / 4.0, height], ui, header, properties);
    self.draw_property_editor(
      [left + width / 4.0, top],
      [width / 4.0 * 3.0, height],
      ui,
      header,
      properties,
    );
  }

  fn draw_property_selector(
    &mut self,
    pos: [f32; 2],
    size: [f32; 2],
    ui: &Ui,
    header: &AssetHeader,
    properties: &mut Vec<Property>,
  ) {
    let w = Window::new(im_str!("Properties"))
      .flags(self.window_flags)
      .resizable(false)
      .collapsible(false)
      .movable(false)
      .position(pos, Condition::Always)
      .size(size, Condition::Always);
    w.build(&ui, || {
      for prop in properties.iter() {
        let active = Some(&prop.meta.name) == self.selected.as_ref().map(|x| &x.name);
        if ui.radio_button_bool(
          &ImString::from(prop.meta.name.to_string(&header.names)),
          active,
        ) && !active
        {
          self.selected = Some(SelectedProperty {
            name: prop.meta.name.clone(),
            dirty: false,
            plugin: EditorPlugin::new(prop),
          });
        }
      }
    });
  }

  fn draw_property_editor(
    &mut self,
    pos: [f32; 2],
    size: [f32; 2],
    ui: &Ui,
    header: &AssetHeader,
    properties: &mut Vec<Property>,
  ) {
    let w = Window::new(im_str!("Property Editor"))
      .flags(self.window_flags)
      .resizable(false)
      .collapsible(false)
      .movable(false)
      .position(pos, Condition::Always)
      .size(size, Condition::Always);
    w.build(&ui, || {
      if let Some(selected) = &mut self.selected {
        if selected.dirty {
          ui.text(ImString::from(format!(
            "*{}",
            selected.name.to_string(&header.names)
          )));
        } else {
          ui.text(ImString::from(selected.name.to_string(&header.names)));
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Save"), [0.0, 0.0]) {
          let index = properties
            .iter()
            .position(|prop| prop.meta.name == selected.name)
            .expect(&format!(
              "Property {} was removed while editing. Report to maintaner",
              selected.name.to_string(&header.names)
            ));
          selected.dirty = false;
          properties[index] = selected.plugin.as_property(selected.name.clone());
        }

        ui.separator();
        let changed = selected.plugin.input(ui, header);
        selected.dirty = selected.dirty || changed;
      } else {
        ui.text("Select a property to edit it");
      }
    });
  }

  pub fn with_flags(self, window_flags: WindowFlags) -> Self {
    Self {
      selected: self.selected,
      window_flags,
    }
  }
}

impl Default for PropertiesEditor {
  fn default() -> Self {
    Self {
      selected: None,
      window_flags: WindowFlags::empty(),
    }
  }
}
