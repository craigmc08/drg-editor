use crate::asset::property::meta::*;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::bindings::*;
use crate::editor::internal::*;

use imgui::*;

pub enum PluginType {
  PluginNone {
    reason: String,
  },
  PluginObject {
    dep: Dependency,
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
  fn to_property(&self, original: &Property) -> Property {
    let name = original.meta.name.clone();
    match self {
      Self::PluginNone { .. } => original.clone(),
      Self::PluginObject { dep } => dep.as_property(name),
      Self::PluginArray { sub_editors, .. } => sub_editors.as_property(name),
      Self::PluginBool { value } => value.as_property(name),
      Self::PluginFloat { value } => value.as_property(name),
      Self::PluginInt { value } => value.as_property(name),
      Self::PluginStr { value } => value.to_string().as_property(name),
    }
  }
}

pub struct EditorPlugin {
  original: Property,
  plugin: PluginType,
}

impl EditorPlugin {
  pub fn new(property: &Property) -> Self {
    let plugin = match &property.value {
      Value::Object(value) => PluginType::PluginObject { dep: value.clone() },
      Value::Array { values } => {
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
        let mut str = ImString::from(value.clone());
        str.reserve(64);
        PluginType::PluginStr { value: str }
      }
      _ => PluginType::PluginNone {
        reason: format!("Unsupported property type {}", property.meta.typ),
      },
    };
    EditorPlugin {
      original: property.clone(),
      plugin,
    }
  }

  /// Returns true if a change was made
  pub fn input(&mut self, ui: &Ui, asset: &Asset) -> bool {
    match &mut self.plugin {
      PluginType::PluginNone { reason } => {
        ui.text(format!(
          "Can't edit {}: {}",
          self.original.meta.name, reason
        ));
        false
      }
      PluginType::PluginObject { dep } => {
        if let Some(new_dep) = input_dependency(ui, "ObjectProperty", asset, dep.clone()) {
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
          let sub_editor = EditorPlugin::default_from_type(*value_type);
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
        prev != *value
      }
      PluginType::PluginInt { value } => {
        let prev = *value;
        ui.input_int(im_str!("Int"), value).build();
        prev != *value
      }
      PluginType::PluginStr { value } => ui.input_text(im_str!("String"), value).build(),
    }
  }

  pub fn default_from_type(typ: PropType) -> Self {
    let (tag, value) = match typ {
      PropType::ObjectProperty => (Tag::Simple(typ), Value::Object(Dependency::uobject())),
      PropType::BoolProperty => (Tag::Bool(false), Value::Bool),
      PropType::IntProperty => (Tag::Simple(typ), Value::Int(0)),
      PropType::FloatProperty => (Tag::Simple(typ), Value::Float(0.0)),
      PropType::StrProperty => (Tag::Simple(typ), Value::Str("".to_string())),
      // TODO ArrayProperty
      // TODO StructProperty
      _ => {
        return Self {
          original: Property {
            meta: Meta::new("", typ, 0),
            tag: Tag::Simple(typ),
            value: Value::Bool,
          },
          plugin: PluginType::PluginNone {
            reason: format!("Can't create new {}", typ),
          },
        };
      }
    };
    Self::new(&Property {
      meta: Meta::new("", typ, 0),
      tag,
      value,
    })
  }

  pub fn save(&self, strct: &mut Properties) {
    if let Some(i) = strct
      .properties
      .iter()
      .position(|prop| prop.meta.name == self.original.meta.name)
    {
      strct.properties[i] = self.plugin.to_property(&self.original);
    }
  }
}

impl AsProperty for EditorPlugin {
  fn prop_type(&self) -> PropType {
    self.plugin.to_property(&self.original).meta.typ
  }
  fn as_tag(&self) -> Tag {
    self.plugin.to_property(&self.original).tag
  }
  fn as_value(&self) -> Value {
    self.plugin.to_property(&self.original).value
  }
}
impl FromProperty for EditorPlugin {
  fn from_property(property: &Property) -> Option<Self> {
    Some(Self::new(property))
  }
}
