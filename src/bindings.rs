use crate::asset::property::meta::*;
use crate::asset::property::prop_type::*;
use crate::asset::*;

/// Represents a value that can be turned into the parts of a property
pub trait AsProperty {
  fn prop_type(&self) -> PropType;
  fn as_tag(&self) -> Tag;
  fn as_value(&self) -> Value;

  fn as_property<T: Into<NameVariant>>(&self, name: T) -> Property {
    Property {
      // Size doesn't matter
      meta: Meta::new(name, self.prop_type(), 0),
      tag: self.as_tag(),
      value: self.as_value(),
    }
  }
}

// Represents a value that can be turned into a property with a simple tag
pub trait AsSimpleProperty {
  fn prop_type() -> PropType;
  fn as_value(&self) -> Value;
}

impl<T> AsProperty for T
where
  T: AsSimpleProperty,
{
  fn prop_type(&self) -> PropType {
    <Self as AsSimpleProperty>::prop_type()
  }
  fn as_tag(&self) -> Tag {
    Tag::Simple(self.prop_type())
  }
  fn as_value(&self) -> Value {
    <Self as AsSimpleProperty>::as_value(self)
  }
}

/// Represents a value that can be created from a property
pub trait FromProperty
where
  Self: std::marker::Sized,
{
  fn from_property(property: &Property) -> Option<Self>;
}

/// Represents a value that can be created from a value
pub trait FromValue
where
  Self: std::marker::Sized,
{
  fn from_value(value: &Value) -> Option<Self>;
}

impl<T: FromValue> FromProperty for T {
  fn from_property(property: &Property) -> Option<Self> {
    Self::from_value(&property.value)
  }
}

impl Property {
  /// This can't be implemented using the standard TryInto trait because it
  /// overlaps with the generic implementation there...
  pub fn try_into<T>(&self) -> Option<T>
  where
    T: FromProperty,
  {
    T::from_property(self)
  }
}

impl Properties {
  /// Get the value of a property by name
  pub fn get<T: FromProperty>(&self, name: &str) -> Option<T> {
    for prop in self.properties.iter() {
      if prop.meta.name == NameVariant::parse(name) {
        return T::from_property(prop);
      }
    }
    None
  }

  /// Set the value of a property by name
  pub fn set<T: AsProperty>(&mut self, name: &str, value: T) -> () {
    let name = NameVariant::parse(name);
    let new_prop = value.as_property(name.clone());
    match self
      .properties
      .iter()
      .position(|prop| prop.meta.name == name)
    {
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
    let class_package = NameVariant::parse(class_package);
    let class = NameVariant::parse(class);
    let name = NameVariant::parse(name);

    // Ensure the base names are imported
    self.names.add(&class_package.name);
    self.names.add(&class.name);
    self.names.add(&name.name);

    match self
      .imports
      .objects
      .iter()
      .position(|object| object.name == name)
    {
      None => {
        // Create a new import
        let outer_index = outer.serialize(&self.imports, &self.exports);
        self.imports.add(class_package, class, name, outer_index);
      }
      Some(_) => {}
    }
  }

  /// Add an imported object to the preloaded dependencies
  pub fn preload(&mut self, dep: Dependency) -> () {
    match self
      .dependencies
      .dependencies
      .iter()
      .position(|d| d.clone() == dep)
    {
      None => {
        // Create a new preload
        self.dependencies.dependencies.push(dep);
      }
      Some(_) => {}
    }
  }

  // List all imports
  pub fn list_imports(&self) -> &Vec<ObjectImport> {
    &self.imports.objects
  }

  /// List the names of all exports
  pub fn list_exports(&self) -> Vec<NameVariant> {
    self
      .exports
      .exports
      .iter()
      .map(|export| export.object_name.clone())
      .collect()
  }

  /// Attempt to borrow an exported Struct by name.
  ///
  /// To borrow the Struct as mutable, see [Self::get_struct_mut]
  pub fn get_struct(&self, name: &str) -> Option<&Properties> {
    let name = NameVariant::parse(name);
    match self
      .exports
      .exports
      .iter()
      .position(|exp| exp.object_name == name)
    {
      None => None,
      Some(i) => Some(&self.structs[i]),
    }
  }

  /// Attempt to borrow as mutable an exported Struct by name.
  pub fn get_struct_mut(&mut self, name: &str) -> Option<&mut Properties> {
    let name = NameVariant::parse(name);
    match self
      .exports
      .exports
      .iter()
      .position(|exp| exp.object_name == name)
    {
      None => None,
      Some(i) => Some(&mut self.structs[i]),
    }
  }
}

//======================
// TRAIT IMPLEMENTATIONS
//======================

impl AsProperty for bool {
  fn prop_type(&self) -> PropType {
    PropType::BoolProperty
  }
  fn as_tag(&self) -> Tag {
    Tag::Bool(*self)
  }
  fn as_value(&self) -> Value {
    Value::Bool
  }
}
impl FromProperty for bool {
  fn from_property(property: &Property) -> Option<Self> {
    match &property.tag {
      Tag::Bool(value) => Some(*value),
      _ => None,
    }
  }
}

impl AsSimpleProperty for i32 {
  fn prop_type() -> PropType {
    PropType::IntProperty
  }
  fn as_value(&self) -> Value {
    Value::Int(*self)
  }
}
impl FromValue for i32 {
  fn from_value(value: &Value) -> Option<Self> {
    match value {
      Value::Int(value) => Some(*value),
      _ => None,
    }
  }
}

impl AsSimpleProperty for f32 {
  fn prop_type() -> PropType {
    PropType::FloatProperty
  }
  fn as_value(&self) -> Value {
    Value::Float(*self)
  }
}
impl FromValue for f32 {
  fn from_value(value: &Value) -> Option<Self> {
    match value {
      Value::Float(value) => Some(*value),
      _ => None,
    }
  }
}

impl<T> AsProperty for Vec<T>
where
  T: AsProperty,
{
  fn prop_type(&self) -> PropType {
    PropType::ArrayProperty
  }
  fn as_tag(&self) -> Tag {
    let inner_type = self[0].prop_type();
    Tag::Array { inner_type }
  }
  fn as_value(&self) -> Value {
    let values: Vec<Value> = self.iter().map(|t| t.as_value()).collect();
    Value::Array { values }
  }
}
impl<T> FromValue for Vec<T>
where
  T: FromValue,
{
  fn from_value(value: &Value) -> Option<Self> {
    match &value {
      Value::Array { values } => Some(values.iter().filter_map(|v| T::from_value(v)).collect()),
      _ => None,
    }
  }
}

impl AsSimpleProperty for Dependency {
  fn prop_type() -> PropType {
    PropType::ObjectProperty
  }
  fn as_value(&self) -> Value {
    Value::Object(self.clone())
  }
}
impl FromValue for Dependency {
  fn from_value(value: &Value) -> Option<Self> {
    match &value {
      Value::Object(value) => Some(value.clone()),
      _ => None,
    }
  }
}
