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
    value_tag: PropertyTag,
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
    match self {
      Self::PluginNone { .. } => original.clone(),
      Self::PluginObject { dep } => dep.as_property(original.name.clone()),
      Self::PluginArray { sub_editors, .. } => {
        vec_as_property_unsafe(sub_editors, original.name.clone())
      }
      Self::PluginBool { value } => value.as_property(original.name.clone()),
      Self::PluginFloat { value } => value.as_property(original.name.clone()),
      Self::PluginInt { value } => value.as_property(original.name.clone()),
      Self::PluginStr { value } => value.to_string().as_property(original.name.clone()),
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
      PropertyValue::ObjectProperty { value } => PluginType::PluginObject { dep: value.clone() },
      PropertyValue::ArrayProperty { values } => {
        if let PropertyTagData::ArrayTag { value_tag } = property.tag_data {
          let mut sub_editors = vec![];
          for editor in values.iter().map(EditorPlugin::new_from_nv) {
            if let Some(editor) = editor {
              sub_editors.push(editor);
            } else {
              return EditorPlugin {
                original: property.clone(),
                plugin: PluginType::PluginNone {
                  reason: format!("Empty value in array"),
                },
              };
            }
          }
          PluginType::PluginArray {
            value_tag,
            sub_editors,
          }
        } else {
          unreachable!()
        }
      }
      PropertyValue::BoolProperty {} => {
        if let PropertyTagData::BoolTag { value } = property.tag_data {
          PluginType::PluginBool { value }
        } else {
          unreachable!()
        }
      }
      PropertyValue::FloatProperty { value } => PluginType::PluginFloat { value: *value },
      PropertyValue::IntProperty { value } => PluginType::PluginInt { value: *value },
      PropertyValue::StrProperty { value } => {
        let mut str = ImString::from(value.clone());
        str.reserve(64);
        PluginType::PluginStr { value: str }
      }
      _ => PluginType::PluginNone {
        reason: format!("Unsupported property type {:?}", property.tag),
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
        ui.text(format!("Can't edit {}: {}", self.original.name, reason));
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
        value_tag,
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
          if let Some(sub_editor) = EditorPlugin::new_from_nv(&NestedValue::new(*value_tag)) {
            sub_editors.push(sub_editor);
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

  fn new_from_nv(nv: &NestedValue) -> Option<Self> {
    match nv {
      NestedValue::Simple { value } => Some(Self::new(&Property {
        // Every field except value doesn't matter
        name: "".into(),
        tag: PropertyTag::ByteProperty,
        size: 0,
        tag_data: PropertyTagData::EmptyTag {
          tag: PropertyTag::ByteProperty,
        },
        value: value.clone(),
      })),
      NestedValue::Complex { value: Some(value) } => Some(Self::new(value)),
      NestedValue::Complex { .. } => None,
    }
  }

  pub fn save(&self, strct: &mut Struct) {
    if let Some(i) = strct
      .properties
      .iter()
      .position(|prop| prop.name == self.original.name)
    {
      strct.properties[i] = self.plugin.to_property(&self.original);
    }
  }
}

impl AsProperty for EditorPlugin {
  fn as_property<T: Into<NameVariant>>(&self, _name: T) -> Property {
    self.plugin.to_property(&self.original)
  }
}
impl FromProperty for EditorPlugin {
  fn from_property(property: &Property) -> Option<Self> {
    Some(Self::new(property))
  }
}
