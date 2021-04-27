use crate::asset::*;

/// Represents a value that can be turned into a named property
pub trait AsProperty {
  fn tag() -> PropertyTag;
  fn as_property(&self, name: &str) -> Property;
  
  fn as_nested_value(&self, name: &str) -> NestedValue {
    let prop = self.as_property(name);
    if Self::tag().is_complex_array_value() {
      NestedValue::Complex { value: Some(prop) }
    } else {
      NestedValue::Simple { value: prop.value }
    }
  }
}

/// Represents a value that can be created from a property
pub trait FromProperty where Self: std::marker::Sized {
  fn from_property(property: &Property) -> Option<Self>;

  /// Assumes that [`Self::from_property`] if, as a nested value, T is a
  /// Simple, then it only cares about `property.value`.
  fn from_nested_value(value: &NestedValue) -> Option<Self> {
    match value {
      NestedValue::Complex { value } => value.as_ref().and_then(|prop| Self::from_property(prop)),
      NestedValue::Simple { value } => Self::from_property(&Property {
        name: "".to_string(),
        name_variant: 0,
        tag: PropertyTag::ByteProperty,
        tag_data: PropertyTagData::EmptyTag { tag: PropertyTag::ByteProperty },
        size: 0,
        value: value.clone(),
      })
    }
  }
}

impl Struct {
  /// Get the value of a property by name
  pub fn get<T: FromProperty>(&self, name: &str) -> Option<T> {
    for prop in self.properties.iter() {
      if prop.name == name {
        return T::from_property(prop)
      }
    }
    None
  }

  /// Set the value of a property by name
  pub fn set<T: AsProperty>(&mut self, name: &str, value: T) -> () {
    let new_prop = value.as_property(name);
    match self.properties.iter().position(|prop| prop.name == name) {
      None => {
        self.properties.push(new_prop);
      }
      Some(i) => {
        self.properties[i] = new_prop;
      }
    }
  }
}

impl Asset {
  /// Import an object into the asset
  /// 
  /// # Arguments
  /// 
  /// * `class_package` - The package of the item to import
  /// * `class` - The class of the item to import
  /// * `name` - The name of the item to import
  /// 
  /// 
  /// # Examples
  /// 
  /// ```
  /// asset.import("/Script/CoreUObject", "Package", "/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun", Dependency::UObject);
  /// asset.import("/Script/FSD", "ItemID", "ID_GrapplingGun", Dependency::Import("/Game/WeaponsNTools/GrapplingGun/ID_Grappling");
  /// ```
  pub fn import(&mut self, class_package: &str, class: &str, name: &str, outer: Dependency) -> () {
    // Ensure the names are imported
    self.names.add(class_package);
    self.names.add(class);
    self.names.add(name);
    
    match self.imports.objects.iter().position(|object| object.name == name) {
      None => {
        // Create a new import
        let outer_index = outer.serialize(&self.imports, &self.exports);
        self.imports.add(class_package, class, name, outer_index);
      }
      Some(_) => { }
    }
  }

  /// Add an imported object to the preloaded dependencies
  pub fn preload(&mut self, dep: Dependency) -> () {
    match self.dependencies.dependencies.iter().position(|d| d.clone() == dep) {
      None => {
        // Create a new preload
        self.dependencies.dependencies.push(dep);
      }
      Some(_) => {}
    }
  }

  /// List the names of all exported Structs
  pub fn list_structs(&self) -> Vec<String> {
    self.exports.exports.iter().map(|export| export.object_name.clone()).collect()
  }

  /// Attempt to borrow an exported Struct by name.
  /// 
  /// To borrow the Struct as mutable, see [Self::get_struct_mut]
  pub fn get_struct(&self, name: &str) -> Option<&Struct> {
    match self.exports.exports.iter().position(|exp| exp.object_name == name) {
      None => None,
      Some(i) => {
        Some(&self.structs[i])
      }
    }
  }

  /// Attempt to borrow as mutable an exported Struct by name.
  pub fn get_struct_mut(&mut self, name: &str) -> Option<&mut Struct> {
    match self.exports.exports.iter().position(|exp| exp.object_name == name) {
      None => None,
      Some(i) => {
        Some(&mut self.structs[i])
      }
    }
  }
}

//======================
// TRAIT IMPLEMENTATIONS
//======================

impl AsProperty for bool {
  fn tag() -> PropertyTag {
    PropertyTag::BoolProperty
  }
  fn as_property(&self, name: &str) -> Property {
    Property {
      name: name.to_string(),
      name_variant: 0,
      tag: Self::tag(),
      size: 0,
      tag_data: PropertyTagData::BoolTag { value: *self },
      value: PropertyValue::BoolProperty {},
    }
  }
}
impl FromProperty for bool {
  fn from_property(property: &Property) -> Option<Self> {
    match property.tag_data {
      PropertyTagData::BoolTag { value } => Some(value),
      _ => None
    }
  }
}

impl AsProperty for u8 {
  fn tag() -> PropertyTag {
    PropertyTag::ByteProperty
  }
  fn as_property(&self, name: &str) -> Property {
    Property {
      name: name.to_string(),
      name_variant: 0,
      tag: Self::tag(),
      size: 1,
      tag_data: PropertyTagData::EmptyTag { tag: Self::tag() },
      value: PropertyValue::ByteProperty { value: *self }
    }
  }
}
impl FromProperty for u8 {
  fn from_property(property: &Property) -> Option<Self> {
    match property.value {
      PropertyValue::ByteProperty { value } => Some(value),
      _ => None
    }
  }
}

impl<T> AsProperty for Vec<T> where T: AsProperty {
  fn tag() -> PropertyTag {
    PropertyTag::ArrayProperty
  }
  fn as_property(&self, name: &str) -> Property {
    let tag_data = PropertyTagData::ArrayTag { value_tag: T::tag() };
    let array: Vec<NestedValue> = self.iter().map(|t| t.as_nested_value(name)).collect();
    let size = array.iter().map(|nv| nv.byte_size()).sum::<usize>() + 4;
    Property {
      name: name.to_string(),
      name_variant: 0,
      tag: Self::tag(),
      size: size as u64,
      tag_data,
      value: PropertyValue::ArrayProperty { values: array }
    }
  }
}
impl<T> FromProperty for Vec<Option<T>> where T: FromProperty {
  fn from_property(property: &Property) -> Option<Self> {
    match &property.value {
      PropertyValue::ArrayProperty { values } => {
        Some(values.iter().map(|nv| T::from_nested_value(nv)).collect())
      }
      _ => None
    }
  }
}

impl AsProperty for Dependency {
  fn tag() -> PropertyTag {
    PropertyTag::ObjectProperty
  }
  fn as_property(&self, name: &str) -> Property {
    Property {
      name: name.to_string(),
      name_variant: 0,
      tag: Self::tag(),
      size: 8,
      tag_data: PropertyTagData::EmptyTag { tag: Self::tag() },
      value: PropertyValue::ObjectProperty { value: self.clone() }
    }
  }
}
impl FromProperty for Dependency {
  fn from_property(property: &Property) -> Option<Self> {
    match &property.value {
      PropertyValue::ObjectProperty { value } => Some(value.clone()),
      _ => None
    }
  }
}
