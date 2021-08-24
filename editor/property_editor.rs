use crate::plugins::*;
use crate::property_creator::*;
use drg::bindings::AsProperty;
use drg::property::prop_type::*;
use drg::*;
use imgui::*;

pub struct PropertiesEditor {
  selected: Option<SelectedProperty>,
  window_flags: WindowFlags,
  creator: Option<PropertyCreator>,
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
    header: &mut AssetHeader,
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
    header: &mut AssetHeader,
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
      // Property creator
      if ui.button(im_str!("Add Property"), [0.0, 0.0]) {
        self.creator = Some(PropertyCreator::new(
          NameVariant::parse("None", &header.names),
          PropType::IntProperty,
          header,
        ));
      }
      match self.creator.take() {
        None => {}
        Some(mut creator) => match creator.draw("New Property", ui, header) {
          EditStatus::Continue => {
            self.creator = Some(creator);
          }
          EditStatus::Cancel => {
            self.creator = None;
          }
          EditStatus::Done => {
            if let Some(prop) = creator.build(header) {
              properties.push(prop);
            }
          }
        },
      }

      // Property list
      let mut to_remove = vec![];
      for (i, prop) in properties.iter().enumerate() {
        let mut active = Some(&prop.meta.name) == self.selected.as_ref().map(|x| &x.name);
        if ui.radio_button_bool(
          &ImString::from(prop.meta.name.to_string(&header.names)),
          active,
        ) && !active
        {
          self.selected = Some(SelectedProperty {
            name: prop.meta.name.clone(),
            dirty: false,
            plugin: EditorPlugin::new(prop, header),
          });
          active = true;
        }
        ui.same_line(0.0);
        // push and pop id so that same named "X" button works for all elements
        let id = ui.push_id(i as i32);
        if ui.button(im_str!("X"), [0.0, 0.0]) {
          to_remove.push(i);
          if active {
            self.selected = None;
          }
        }
        id.pop(ui);
      }

      // Iterate backwards to prevent invalidating indices
      for i in to_remove.into_iter().rev() {
        properties.remove(i);
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
          properties[index] = selected.plugin.as_property(selected.name.clone(), header);
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
      creator: None,
    }
  }
}

impl Default for PropertiesEditor {
  fn default() -> Self {
    Self {
      selected: None,
      window_flags: WindowFlags::empty(),
      creator: None,
    }
  }
}
