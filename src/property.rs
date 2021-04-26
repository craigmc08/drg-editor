use crate::file_summary::*;
use crate::name_map::*;
use crate::object_imports::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub enum PropertyValue {
  BoolProperty {
    value: bool,
  },
  ByteProperty {
    value: u8,
  },
  // EnumProperty, TODO
  // TextProperty, TODO
  // StrProperty, TODO
  // NameProperty, TODO
  ArrayProperty {
    value_tag: String,
    values: Vec<PropertyValue>,
  },
  // MapProperty, TODO
  ObjectProperty {
    value: String,
  },
  // StructProperty, TODO
  // DebugProperty, TODO
  // SetProperty, TODO
  Int8Property {
    value: i8,
  },
  Int16Property {
    value: i16,
  },
  IntProperty {
    value: i32,
  },
  Int64Property {
    value: i64,
  },
  UInt16Property {
    value: u16,
  },
  UInt32Property {
    value: u32,
  },
  UInt64Property {
    value: u64,
  },
  FloatProperty {
    value: f32,
  },
  DoubleProperty {
    value: f64,
  },
  // WeakObjectProperty, TODO
  // LazyObjectProperty, TODO
  SoftObjectProperty {
    object_name: String,
    unk1: u32,
  },
  // DelegateProperty, TODO
  // MulticastDelegateProperty, TODO
  // InterfaceProperty, TODO
  // FieldPathProperty, TODO
  // AssetObjectProperty, TODO
}

#[derive(Debug)]
pub struct Property {
  pub name: String, // u64 index into name_map
  pub tag: String,  // u64 index into name map
  // 1 byte of padding?
  pub size: u64, // size of property
  pub value: PropertyValue,
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
        let mut values = vec![];
        for _ in 0..length {
          let value = Self::read(rdr, &value_tag, names, imports)?;
          values.push(value);
        }
        Ok(Self::ArrayProperty { value_tag, values })
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
      "FloatProperty" => Ok(Self::FloatProperty {
        value: rdr.read_f32::<LittleEndian>().unwrap(),
      }),
      "DoubleProperty" => Ok(Self::DoubleProperty {
        value: rdr.read_f64::<LittleEndian>().unwrap(),
      }),
      "SoftObjectProperty" => {
        let object_name = names.read_name(rdr, "SoftObjectProperty object_name")?;
        let unk1 = read_u32(rdr);
        Ok(Self::SoftObjectProperty { object_name, unk1 })
      }
      _ => Err(format!("Unknown tag type {}", tag)),
    }
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports) -> () {
    match self {
      Self::BoolProperty { value } => write_bool(curs, *value),
      Self::ByteProperty { value } => curs.write_u8(*value).unwrap(),
      Self::ArrayProperty { value_tag, values } => {
        let value_tag_n = names
          .get_name_obj(value_tag)
          .expect("Invalid ArrayProperty value_tag");
        curs.write_u64::<LittleEndian>(value_tag_n.index).unwrap();
        curs.write(&[0]).unwrap(); // weird padding
        write_u32(curs, values.len() as u32);
        for value in values.iter() {
          value.write(curs, names, imports);
        }
      }
      Self::ObjectProperty { value } => {
        let value = imports
          .serialized_index_of(value)
          .expect("Invalid ObjectProperty value");
        write_u32(curs, value);
      }
      Self::Int8Property { value } => curs.write_i8(*value).unwrap(),
      Self::Int16Property { value } => curs.write_i16::<LittleEndian>(*value).unwrap(),
      Self::IntProperty { value } => curs.write_i32::<LittleEndian>(*value).unwrap(),
      Self::Int64Property { value } => curs.write_i64::<LittleEndian>(*value).unwrap(),
      Self::UInt16Property { value } => curs.write_u16::<LittleEndian>(*value).unwrap(),
      Self::UInt32Property { value } => curs.write_u32::<LittleEndian>(*value).unwrap(),
      Self::UInt64Property { value } => curs.write_u64::<LittleEndian>(*value).unwrap(),
      Self::FloatProperty { value } => curs.write_f32::<LittleEndian>(*value).unwrap(),
      Self::DoubleProperty { value } => curs.write_f64::<LittleEndian>(*value).unwrap(),
      Self::SoftObjectProperty { object_name, unk1 } => {
        let object_name_n = names
          .get_name_obj(object_name)
          .expect("Invalid SoftObjectProperty object_name");
        curs.write_u64::<LittleEndian>(object_name_n.index).unwrap();
        write_u32(curs, *unk1);
      }
    }
  }

  pub fn byte_size(&self) -> usize {
    match self {
      Self::BoolProperty { .. } => 4,
      Self::ByteProperty { .. } => 1,
      Self::ArrayProperty { values, .. } => {
        // value_tag index + u32 size = 12, values
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
    let name = names.read_name(rdr, "Property name")?;
    if &name == "None" {
      return Ok(None);
    }

    let tag = names.read_name(rdr, "Property tag")?;
    let size = rdr.read_u64::<LittleEndian>().unwrap();
    if tag != "ArrayProperty" {
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

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports) -> () {
    let name = names
      .get_name_obj(&self.name)
      .expect("Invalid Property name");
    let tag = names.get_name_obj(&self.tag).expect("Invalid Property tag");
    curs.write_u64::<LittleEndian>(name.index).unwrap();
    curs.write_u64::<LittleEndian>(tag.index).unwrap();
    curs.write_u64::<LittleEndian>(self.size).unwrap();
    if self.tag != "ArrayProperty" {
      // See note in self.read, this is bad
      curs.write(&[0]);
    }
    self.value.write(curs, names, imports);
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

  pub fn write_uexp(
    props: &Vec<Property>,
    curs: &mut Cursor<Vec<u8>>,
    summary: &FileSummary,
    names: &NameMap,
    imports: &ObjectImports,
  ) -> () {
    for prop in props.iter() {
      prop.write(curs, names, imports);
    }

    // Write none property
    let none = names
      .get_name_obj("None")
      .expect("None should be in names map");
    curs.write_u64::<LittleEndian>(none.index).unwrap();
    write_u32(curs, 0);

    // Write ending tag
    curs.write(&summary.tag);
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
