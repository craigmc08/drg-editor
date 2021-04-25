use crate::name_map::*;
use crate::object_imports::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub enum PropertyValue {
  BoolProperty { value: bool },
  ByteProperty { value: u8 },
  // EnumProperty, TODO
  // TextProperty, TODO
  // StrProperty, TODO
  // NameProperty, TODO
  ArrayProperty { values: Vec<PropertyValue> },
  // MapProperty, TODO
  ObjectProperty { value: String },
  // StructProperty, TODO
  // DebugProperty, TODO
  // SetProperty, TODO
  Int8Property { value: i8 },
  Int16Property { value: i16 },
  IntProperty { value: i32 },
  Int64Property { value: i64 },
  UInt16Property { value: u16 },
  UInt32Property { value: u32 },
  UInt64Property { value: u64 },
  FloatProperty { value: f32 },
  DoubleProperty { value: f64 },
  // WeakObjectProperty, TODO
  // LazyObjectProperty, TODO
  SoftObjectProperty { object_name: String, unk1: u32 },
  // DelegateProperty, TODO
  // MulticastDelegateProperty, TODO
  // InterfaceProperty, TODO
  // FieldPathProperty, TODO
  // AssetObjectProperty, TODO
}

#[derive(Debug)]
pub struct Property {
  name: String, // u64 index into name_map
  tag: String,  // u64 index into name map
  // 1 byte of padding?
  size: u64, // size of property
  value: PropertyValue,
}

impl PropertyValue {
  fn read(
    rdr: &mut Cursor<Vec<u8>>,
    tag: &str,
    names: &NameMap,
    imports: &ObjectImports,
  ) -> Result<Self, String> {
    match tag {
      "BoolProperty" => Ok(Self::BoolProperty {
        value: read_bool(rdr),
      }),
      "ByteProperty" => Ok(Self::ByteProperty {
        value: rdr.read_u8().unwrap(),
      }),
      "ArrayProperty" => {
        let value_tag = names.read_name(rdr, "ArrayProperty value_tag")?;
        rdr.consume(1); // padding is after value tag
        let length = read_u32(rdr);
        println!("Array of {} [{}]", value_tag, length);
        let mut values = vec![];
        for _ in 0..length {
          println!("item {:04X}", rdr.position());
          let value = Self::read(rdr, &value_tag, names, imports)?;
          values.push(value);
        }
        Ok(Self::ArrayProperty { values })
      }
      "ObjectProperty" => {
        let value = imports.read_import(rdr, "ObjectProperty value")?;
        Ok(Self::ObjectProperty { value })
      }
      "Int8Property" => Ok(Self::Int8Property {
        value: rdr.read_i8().unwrap(),
      }),
      "Int16Property" => Ok(Self::Int16Property {
        value: rdr.read_i16::<LittleEndian>().unwrap(),
      }),
      "IntProperty" => Ok(Self::IntProperty {
        value: rdr.read_i32::<LittleEndian>().unwrap(),
      }),
      "Int64Property" => Ok(Self::Int64Property {
        value: rdr.read_i64::<LittleEndian>().unwrap(),
      }),
      "UInt16Property" => Ok(Self::UInt16Property {
        value: rdr.read_u16::<LittleEndian>().unwrap(),
      }),
      "UInt32Property" => Ok(Self::UInt32Property {
        value: read_u32(rdr),
      }),
      "UInt64Property" => Ok(Self::UInt64Property {
        value: rdr.read_u64::<LittleEndian>().unwrap(),
      }),
      "SoftObjectProperty" => {
        let object_name = names.read_name(rdr, "SoftObjectProperty object_name")?;
        let unk1 = read_u32(rdr);
        Ok(Self::SoftObjectProperty { object_name, unk1 })
      }
      _ => Err(format!("Unknown tag type {}", tag)),
    }
  }

  pub fn byte_size(&self) -> usize {
    match self {
      Self::BoolProperty { .. } => 4,
      Self::ByteProperty { .. } => 1,
      Self::ArrayProperty { values } => {
        // tag index + u32 size = 12, values
        12 + values.into_iter().map(|x| x.byte_size()).sum::<usize>()
      }
      Self::ObjectProperty { .. } => 4,
      Self::Int8Property { .. } => 1,
      Self::Int16Property { .. } => 2,
      Self::IntProperty { .. } => 4,
      Self::Int64Property { .. } => 8,
      Self::UInt16Property { .. } => 2,
      Self::UInt32Property { .. } => 4,
      Self::UInt64Property { .. } => 8,
      Self::FloatProperty { .. } => 4,
      Self::DoubleProperty { .. } => 8,
      Self::SoftObjectProperty { .. } => 12,
    }
  }
}

impl Property {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
  ) -> Result<Option<Self>, String> {
    println!("{:04X}", rdr.position());
    let name = names.read_name(rdr, "Property name")?;
    if &name == "None" {
      return Ok(None);
    }

    let tag = names.read_name(rdr, "Property tag")?;
    let size = rdr.read_u64::<LittleEndian>().unwrap();
    if &tag != "ArrayProperty" {
      // This shouldn't happen for arrayproperties, because idk
      rdr.consume(1); // weird 1 byte of padding for some reason
    }
    let value = PropertyValue::read(rdr, &tag, names, imports)?;
    // if value.byte_size() != size.try_into().unwrap() {
    //   return Err(format!(
    //     "Property size {} does not match actual size {}",
    //     size,
    //     value.byte_size()
    //   ));
    // }
    return Ok(Some(Property {
      name,
      tag,
      size,
      value,
    }));
  }

  pub fn read_uexp(
    rdr: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
  ) -> Result<Vec<Self>, String> {
    let mut properties = vec![];
    // Read properties until summary.tag is seen
    loop {
      let property = Self::read(rdr, names, imports)?;
      match property {
        None => return Ok(properties),
        Some(prop) => {
          properties.push(prop);
        }
      }
    }
  }

  pub fn byte_size(&self) -> usize {
    // 8 bytes each for name, tag, and size, 1 byte for random padding and obviously size bytes for the value
    25 + self.value.byte_size()
  }

  pub fn struct_size(properties: &Vec<Property>) -> usize {
    // Size of properties plus size of None property at end of the struct
    properties.iter().map(|x| x.byte_size()).sum::<usize>() + 12
  }
}
