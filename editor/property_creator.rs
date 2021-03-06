use crate::internal::*;
use drg::asset::property::meta::*;
use drg::asset::property::prop_type::*;
use drg::asset::property::*;
use drg::asset::*;
use imgui::*;

#[derive(Debug, PartialEq)]
pub enum EditStatus {
  Continue,
  Cancel,
  Done,
}

pub enum ValueCreator {
  WithDefault { for_type: PropType, value: Value },
  Array { inner_type: PropType },
  NoCreator { for_type: PropType },
}

pub struct PropertyCreator {
  name: ImString,
  for_type: PropType,
  creator: ValueCreator,
}

const DEFAULT_PROP_TYPE: PropType = PropType::IntProperty;

impl ValueCreator {
  pub fn new(for_type: PropType, header: &AssetHeader) -> Self {
    match for_type {
      PropType::ArrayProperty => ValueCreator::Array {
        inner_type: DEFAULT_PROP_TYPE,
      },

      PropType::ObjectProperty => ValueCreator::WithDefault {
        for_type,
        value: Value::Object(Reference::uobject()),
      },
      PropType::SoftObjectProperty => ValueCreator::WithDefault {
        for_type,
        value: Value::SoftObject {
          object_name: NameVariant::parse("None", &header.names),
          parent: Reference::uobject(),
        },
      },
      PropType::IntProperty => ValueCreator::WithDefault {
        for_type,
        value: Value::Int(0),
      },
      PropType::FloatProperty => ValueCreator::WithDefault {
        for_type,
        value: Value::Float(0.0),
      },
      PropType::StrProperty => ValueCreator::WithDefault {
        for_type,
        value: Value::Str("".to_string()),
      },

      _ => ValueCreator::NoCreator { for_type },
    }
  }

  /// Returns `true` if `ValueCreator::build` is ready to be called
  pub fn draw(&mut self, ui: &Ui, _header: &AssetHeader, draw_buttons: bool) -> EditStatus {
    match self {
      Self::WithDefault { .. } => EditStatus::Done,
      Self::Array { inner_type } => {
        input_prop_type(ui, "Element Type", inner_type);
        if draw_buttons {
          creator_buttons(ui)
        } else {
          EditStatus::Continue
        }
      }

      Self::NoCreator { .. } => EditStatus::Done,
    }
  }

  // Same as `ValueCreator::draw` but makes a dialog window for itself
  pub fn draw_dialog(&mut self, label: &str, ui: &Ui, header: &AssetHeader) -> EditStatus {
    let mut status = EditStatus::Continue;

    ui.open_popup(&ImString::new(label));

    ui.popup_modal(&ImString::new(label))
      .always_auto_resize(true)
      .build(|| {
        status = self.draw(ui, header, true);
      });
    status
  }

  pub fn build(self, _header: &AssetHeader) -> Option<(Tag, Value)> {
    match self {
      Self::WithDefault { value, for_type } => Some((Tag::Simple(for_type), value)),
      Self::Array { inner_type } => Some((
        Tag::Array { inner_type },
        Value::Array {
          meta_tag: None, // TODO: This is wrong for some arrays. Fix when adding struct creator
          values: vec![],
        },
      )),

      Self::NoCreator { .. } => None,
    }
  }

  pub fn for_type(&self) -> PropType {
    match self {
      Self::Array { .. } => PropType::ArrayProperty,
      Self::WithDefault { for_type, .. } => *for_type,
      Self::NoCreator { for_type } => *for_type,
    }
  }
}

impl PropertyCreator {
  pub fn new(name: NameVariant, for_type: PropType, header: &AssetHeader) -> Self {
    Self {
      name: ImString::from(name.to_string(&header.names)),
      for_type,
      creator: ValueCreator::new(for_type, header),
    }
  }

  /// Returns `true` if `PropertyCreator::build` is ready to be called
  pub fn draw(&mut self, label: &str, ui: &Ui, header: &AssetHeader) -> EditStatus {
    let mut status = EditStatus::Continue;

    ui.open_popup(&ImString::new(label));

    ui.popup_modal(&ImString::new(label))
      .always_auto_resize(true)
      .build(|| {
        ui.input_text(im_str!("Name"), &mut self.name)
          .resize_buffer(true)
          .build();

        if input_prop_type(ui, "Type", &mut self.for_type) {
          self.creator = ValueCreator::new(self.for_type, header);
        }
        status = self.creator.draw(ui, header, false);

        if status != EditStatus::Cancel {
          status = creator_buttons(ui);
        }
      });
    status
  }

  pub fn build(self, header: &mut AssetHeader) -> Option<Property> {
    let name = NameVariant::parse_and_add(self.name.to_str(), &mut header.names);
    let for_type = self.for_type;
    self.creator.build(header).map(|(tag, value)| {
      let meta = Meta::new(name, for_type, 0);
      Property { meta, tag, value }
    })
  }
}

fn creator_buttons(ui: &Ui) -> EditStatus {
  if ui.button(im_str!("Done"), [0.0, 0.0]) {
    EditStatus::Done
  } else if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
    EditStatus::Cancel
  } else {
    EditStatus::Continue
  }
}
