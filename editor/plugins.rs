use crate::internal::*;
use crate::property_creator::*;
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
  PluginSoftObject {
    object_name: NameVariant,
    parent: Reference,
  },
  PluginArray {
    value_type: PropType,
    sub_editors: Vec<EditorPlugin>,
    value_creator: Option<ValueCreator>,
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

impl AsProperty for PluginType {
  fn prop_type(&self) -> PropType {
    match self {
      PluginType::PluginNone { original, .. } => original.meta.typ,
      PluginType::PluginObject { .. } => PropType::ObjectProperty,
      PluginType::PluginSoftObject { .. } => PropType::SoftObjectProperty,
      PluginType::PluginArray { .. } => PropType::ArrayProperty,
      PluginType::PluginBool { .. } => PropType::BoolProperty,
      PluginType::PluginFloat { .. } => PropType::FloatProperty,
      PluginType::PluginInt { .. } => PropType::IntProperty,
      PluginType::PluginStr { .. } => PropType::StrProperty,
    }
  }
  fn as_tag(&self, _: &AssetHeader) -> Tag {
    match self {
      PluginType::PluginNone { original, .. } => original.tag.clone(),
      PluginType::PluginBool { value } => Tag::Bool(*value),
      PluginType::PluginArray { value_type, .. } => Tag::Array {
        inner_type: *value_type,
      },
      _ => Tag::Simple(self.prop_type()),
    }
  }
  fn as_value(&self, header: &AssetHeader) -> Value {
    match self {
      PluginType::PluginNone { original, .. } => original.value.clone(),
      PluginType::PluginObject { dep } => dep.as_value(header),
      PluginType::PluginSoftObject {
        object_name,
        parent,
      } => Value::SoftObject {
        object_name: object_name.clone(),
        parent: parent.clone(),
      },
      PluginType::PluginArray { sub_editors, .. } => sub_editors.as_value(header),
      PluginType::PluginBool { value } => value.as_value(header),
      PluginType::PluginFloat { value } => value.as_value(header),
      PluginType::PluginInt { value } => value.as_value(header),
      PluginType::PluginStr { value } => value.to_string().as_value(header),
    }
  }
}

pub struct EditorPlugin {
  name: NameVariant,
  plugin: PluginType,
}

impl EditorPlugin {
  pub fn new(property: &Property, header: &AssetHeader) -> Self {
    let plugin = match &property.value {
      Value::Object(value) => PluginType::PluginObject { dep: value.clone() },
      Value::SoftObject {
        object_name,
        parent,
      } => PluginType::PluginSoftObject {
        object_name: object_name.clone(),
        parent: parent.clone(),
      },
      Value::Array { values, .. } => {
        if let Tag::Array { inner_type } = property.tag {
          let mut sub_editors = vec![];
          for editor in values.iter().map(|v| {
            let mut sub_meta = property.meta.clone();
            sub_meta.typ = inner_type;
            Self::new(
              &Property {
                meta: sub_meta,
                tag: property.tag.clone(),
                value: v.clone(),
              },
              header,
            )
          }) {
            sub_editors.push(editor);
          }
          PluginType::PluginArray {
            value_type: inner_type,
            sub_editors,
            value_creator: None,
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
  pub fn input(&mut self, ui: &Ui, header: &AssetHeader) -> bool {
    match &mut self.plugin {
      PluginType::PluginNone { reason, .. } => {
        ui.text(format!(
          "Can't edit {}: {}",
          self.name.to_string(&header.names),
          reason
        ));
        false
      }
      PluginType::PluginObject { dep } => {
        if let Some(new_dep) = input_dependency(ui, "ObjectProperty", &header, dep.clone()) {
          *dep = new_dep;
          true
        } else {
          false
        }
      }
      PluginType::PluginSoftObject {
        object_name,
        parent,
      } => {
        let object_name_changed = if let Some(new_object_name) =
          input_name_variant(ui, "Object Name", header, object_name.clone())
        {
          *object_name = new_object_name;
          true
        } else {
          false
        };
        let parent_changed =
          if let Some(new_parent) = input_dependency(ui, "Parent", header, parent.clone()) {
            *parent = new_parent;
            true
          } else {
            false
          };
        object_name_changed || parent_changed
      }
      PluginType::PluginArray {
        value_type,
        sub_editors,
        value_creator,
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

          changed = changed || editor.input(ui, header);

          id.pop(ui);
        }

        // Iterate backwards to prevent invalidating indices
        for i in to_remove.into_iter().rev() {
          sub_editors.remove(i);
        }

        // Add button
        if ui.button(im_str!("Add Element"), [0.0, 0.0]) {
          *value_creator = Some(ValueCreator::new(*value_type, header));
        }

        if let Some(mut a_value_creator) = value_creator.take() {
          match a_value_creator.draw_dialog("New Array Element", ui, header) {
            EditStatus::Continue => *value_creator = Some(a_value_creator),
            EditStatus::Cancel => {}
            EditStatus::Done => {
              let for_type = a_value_creator.for_type();
              match a_value_creator.build(header) {
                None => {
                  // TODO how to handle case of no value creator
                  panic!("No value creator for type");
                }
                Some((tag, value)) => {
                  let prop = Property {
                    meta: Meta::new(NameVariant::parse("None", &header.names), for_type, 0),
                    tag,
                    value,
                  };
                  sub_editors.push(Self::new(&prop, header));
                  changed = true;
                }
              }
            }
          }
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
}

impl AsProperty for EditorPlugin {
  fn prop_type(&self) -> PropType {
    self.plugin.prop_type()
  }
  fn as_tag(&self, header: &AssetHeader) -> Tag {
    self.plugin.as_tag(header)
  }
  fn as_value(&self, header: &AssetHeader) -> Value {
    self.plugin.as_value(header)
  }
}
impl FromProperty for EditorPlugin {
  fn from_property(property: &Property, header: &AssetHeader) -> Option<Self> {
    Some(Self::new(property, header))
  }
}
