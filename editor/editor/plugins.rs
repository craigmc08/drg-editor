use crate::editor::internal::*;
use drg::asset::property::meta::*;
use drg::asset::property::prop_type::*;
use drg::asset::*;
use drg::bindings::*;

use imgui::*;

pub enum PluginType {
  PluginNone {
    reason: String,
    original: Property,
  },
  PluginObject {
    dep: Reference,
  },
  PluginArray {
    value_type: PropType,
    sub_editors: Vec<EditorPlugin>,
  },
  PluginBool {
    value: bool,
  },
  PluginFloat {
    value: f32,
  },
  PluginInt {
    value: i32,
  },
  PluginStr {
    value: ImString,
  },
}

impl PluginType {
  fn to_property(&self, name: &NameVariant) -> Property {
    let name = name.clone();
    match self {
      Self::PluginNone { original, .. } => original.clone(),
      Self::PluginObject { dep } => dep.as_property(name),
      Self::PluginArray { sub_editors, .. } => sub_editors.as_property(name),
      Self::PluginBool { value } => value.as_property(name),
      Self::PluginFloat { value } => value.as_property(name),
      Self::PluginInt { value } => value.as_property(name),
      Self::PluginStr { value } => value.to_string().as_property(name),
    }
  }
}

impl AsProperty for PluginType {
  fn prop_type(&self) -> PropType {
    match self {
      PluginType::PluginNone { original, .. } => original.meta.typ,
      PluginType::PluginObject { .. } => PropType::ObjectProperty,
      PluginType::PluginArray { .. } => PropType::ArrayProperty,
      PluginType::PluginBool { .. } => PropType::BoolProperty,
      PluginType::PluginFloat { .. } => PropType::FloatProperty,
      PluginType::PluginInt { .. } => PropType::IntProperty,
      PluginType::PluginStr { .. } => PropType::StrProperty,
    }
  }
  fn as_tag(&self) -> Tag {
    match self {
      PluginType::PluginNone { original, .. } => original.tag.clone(),
      PluginType::PluginBool { value } => Tag::Bool(*value),
      PluginType::PluginArray { value_type, .. } => Tag::Array {
        inner_type: *value_type,
      },
      _ => Tag::Simple(self.prop_type()),
    }
  }
  fn as_value(&self) -> Value {
    match self {
      PluginType::PluginNone { original, .. } => original.value.clone(),
      PluginType::PluginObject { dep } => dep.as_value(),
      PluginType::PluginArray { sub_editors, .. } => sub_editors.as_value(),
      PluginType::PluginBool { value } => value.as_value(),
      PluginType::PluginFloat { value } => value.as_value(),
      PluginType::PluginInt { value } => value.as_value(),
      PluginType::PluginStr { value } => value.to_string().as_value(),
    }
  }
}

pub struct EditorPlugin {
  name: NameVariant,
  plugin: PluginType,
}

impl EditorPlugin {
  pub fn new(property: &Property) -> Self {
    let plugin = match &property.value {
      Value::Object(value) => PluginType::PluginObject { dep: value.clone() },
      Value::Array { values, .. } => {
        if let Tag::Array { inner_type } = property.tag {
          let mut sub_editors = vec![];
          // let sub_editors =
          for editor in values.iter().map(|v| {
            Self::new(&Property {
              meta: property.meta.clone(),
              tag: property.tag.clone(),
              value: v.clone(),
            })
          }) {
            sub_editors.push(editor);
          }
          PluginType::PluginArray {
            value_type: inner_type,
            sub_editors,
          }
        } else {
          unreachable!()
        }
      }
      Value::Bool {} => {
        if let Tag::Bool(value) = property.tag {
          PluginType::PluginBool { value }
        } else {
          unreachable!()
        }
      }
      Value::Float(value) => PluginType::PluginFloat { value: *value },
      Value::Int(value) => PluginType::PluginInt { value: *value },
      Value::Str(value) => {
        let str = ImString::from(value.clone());
        PluginType::PluginStr { value: str }
      }
      _ => PluginType::PluginNone {
        original: property.clone(),
        reason: format!("Unsupported property type {}", property.meta.typ),
      },
    };
    EditorPlugin {
      name: property.meta.name.clone(),
      plugin,
    }
  }

  /// Returns true if a change was made
  pub fn input(&mut self, ui: &Ui, asset: &Asset) -> bool {
    match &mut self.plugin {
      PluginType::PluginNone { reason, .. } => {
        ui.text(format!(
          "Can't edit {}: {}",
          self.name.to_string(&asset.header.names),
          reason
        ));
        false
      }
      PluginType::PluginObject { dep } => {
        if let Some(new_dep) = input_dependency(ui, "ObjectProperty", &asset.header, dep.clone()) {
          *dep = new_dep;
          true
        } else {
          false
        }
      }
      PluginType::PluginArray {
        value_type,
        sub_editors,
      } => {
        let mut changed = false;

        // TODO: does moving array elements need to be added?
        let mut to_remove = vec![];
        for (i, editor) in sub_editors.iter_mut().enumerate() {
          let id = ui.push_id(&i.to_string());

          ui.text(format!("Element {}", i));
          ui.same_line(0.0);
          if ui.button(im_str!("X"), [0.0, 0.0]) {
            changed = true;
            to_remove.push(i);
          }

          changed = changed || editor.input(ui, asset);

          id.pop(ui);
        }

        // Iterate backwards to prevent invalidating indices
        for i in to_remove.into_iter().rev() {
          sub_editors.remove(i);
        }

        // Add button
        if ui.button(im_str!("Add Element"), [0.0, 0.0]) {
          changed = true;
          let sub_editor = EditorPlugin::default_from_type(*value_type, &asset.header);
          sub_editors.push(sub_editor);
        }

        changed
      }
      PluginType::PluginBool { value } => {
        let label = if *value { "Yes" } else { "No" };
        ui.checkbox(&ImString::from(label.to_string()), value)
      }
      PluginType::PluginFloat { value } => {
        let prev = *value;
        ui.input_float(im_str!("Float"), value).build();
        (prev - *value).abs() > f32::EPSILON
      }
      PluginType::PluginInt { value } => {
        let prev = *value;
        ui.input_int(im_str!("Int"), value).build();
        prev != *value
      }
      PluginType::PluginStr { value } => ui
        .input_text(im_str!("String"), value)
        .resize_buffer(true)
        .build(),
    }
  }

  pub fn default_from_type(typ: PropType, header: &AssetHeader) -> Self {
    let (tag, value) = match typ {
      PropType::ObjectProperty => (Tag::Simple(typ), Value::Object(Reference::uobject())),
      PropType::BoolProperty => (Tag::Bool(false), Value::Bool),
      PropType::IntProperty => (Tag::Simple(typ), Value::Int(0)),
      PropType::FloatProperty => (Tag::Simple(typ), Value::Float(0.0)),
      PropType::StrProperty => (Tag::Simple(typ), Value::Str("".to_string())),
      // TODO ArrayProperty
      // TODO StructProperty
      _ => {
        // TODO better way to handle this error besides panic!ing?
        panic!("Can't create default property editor for {}", typ)
      }
    };
    Self::new(&Property {
      meta: Meta::new(NameVariant::new("", 0, &header.names), typ, 0),
      tag,
      value,
    })
  }

  pub fn save(&self, strct: &mut Properties) {
    if let Some(i) = strct
      .properties
      .iter()
      .position(|prop| prop.meta.name == self.name)
    {
      strct.properties[i] = self.plugin.to_property(&self.name);
    }
  }
}

impl AsProperty for EditorPlugin {
  fn prop_type(&self) -> PropType {
    self.plugin.prop_type()
  }
  fn as_tag(&self) -> Tag {
    self.plugin.as_tag()
  }
  fn as_value(&self) -> Value {
    self.plugin.as_value()
  }
}
impl FromProperty for EditorPlugin {
  fn from_property(property: &Property) -> Option<Self> {
    Some(Self::new(property))
  }
}
