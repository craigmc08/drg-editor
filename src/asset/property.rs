use crate::asset::*;
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
    value_tag_variant: u32,
    values: Vec<PropertyValue>,
  },
  // MapProperty, TODO
  ObjectProperty {
    value: Dependency,
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
    object_name_variant: u32,
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
  pub name: String, // u632 index into name_map
  pub name_variant: u32,
  pub tag: String,  // u32 index into name map
  pub tag_variant: u32,
  // 1 byte of padding?
  // 8 bytes for size
  pub value: PropertyValue,
}

#[derive(Debug)]
pub struct Struct {
  pub properties: Vec<Property>,
  pub extra: Vec<u8>, // extra unknown info after serial_size for export property
}

impl PropertyValue {
  fn read(
    rdr: &mut Cursor<Vec<u8>>,
    tag: &str,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<Self, String> {
    match tag {
      "BoolProperty" => Ok(Self::BoolProperty {
        value: read_bool(rdr),
      }),
      "ByteProperty" => Ok(Self::ByteProperty {
        value: rdr.read_u8().unwrap(),
      }),
      "ArrayProperty" => {
        let (value_tag, value_tag_variant) = names.read_name_with_variant(rdr, "ArrayProperty value_tag")?;
        rdr.consume(1); // padding is after value tag
        let length = read_u32(rdr);
        let mut values = vec![];
        for _ in 0..length {
          let value = Self::read(rdr, &value_tag, names, imports, exports)?;
          values.push(value);
        }
        Ok(Self::ArrayProperty { value_tag, value_tag_variant, values })
      }
      "ObjectProperty" => {
        let value = Dependency::read(rdr, imports, exports)?;
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
        let (object_name, object_name_variant) = names.read_name_with_variant(rdr, "SoftObjectProperty object_name")?;
        let unk1 = read_u32(rdr);
        Ok(Self::SoftObjectProperty { object_name, object_name_variant, unk1 })
      }
      _ => Err(format!("Unknown tag type {} at {:04X}", tag, rdr.position())),
    }
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> () {
    match self {
      Self::BoolProperty { value } => write_bool(curs, *value),
      Self::ByteProperty { value } => curs.write_u8(*value).unwrap(),
      Self::ArrayProperty { value_tag, value_tag_variant, values } => {
        let value_tag_n = names
          .get_name_obj(value_tag)
          .expect("Invalid ArrayProperty value_tag");
        curs.write_u32::<LittleEndian>(value_tag_n.index).unwrap();
        write_u32(curs, *value_tag_variant);
        curs.write(&[0]).unwrap(); // weird padding
        write_u32(curs, values.len() as u32);
        for value in values.iter() {
          value.write(curs, names, imports, exports);
        }
      }
      Self::ObjectProperty { value } => {
        value.write(curs, imports, exports);
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
      Self::SoftObjectProperty { object_name, object_name_variant, unk1 } => {
        let object_name_n = names
          .get_name_obj(object_name)
          .expect("Invalid SoftObjectProperty object_name");
        curs.write_u32::<LittleEndian>(object_name_n.index).unwrap();
        write_u32(curs, *object_name_variant);
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

  pub fn value_size(&self) -> usize {
    match self {
      Self::BoolProperty { .. } => 4,
      Self::ByteProperty { .. } => 1,
      Self::ArrayProperty { values, .. } => {
        // u32 size = 4, values
        4 + values.into_iter().map(|x| x.byte_size()).sum::<usize>()
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
    exports: &ObjectExports,
  ) -> Result<Option<Self>, String> {
    let (name, name_variant) = names.read_name_with_variant(rdr, "Property name")?;
    if &name == "None" {
      return Ok(None);
    }

    let (tag, tag_variant) = names.read_name_with_variant(rdr, "Property tag")?;
    let _size = rdr.read_u64::<LittleEndian>().unwrap();
    if tag != "ArrayProperty" {
      // This shouldn't happen for arrayproperties, because idk
      rdr.consume(1); // weird 1 byte of padding for some reason
    }
    let value = PropertyValue::read(rdr, &tag, names, imports, exports)?;
    // if value.byte_size() != size.try_into().unwrap() {
    //   return Err(format!(
    //     "Property size {} does not match actual size {}",
    //     size,
    //     value.byte_size()
    //   ));
    // }
    return Ok(Some(Property {
      name,
      name_variant,
      tag,
      tag_variant,
      value,
    }));
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> () {
    let name = names
      .get_name_obj(&self.name)
      .expect("Invalid Property name");
    let tag = names.get_name_obj(&self.tag).expect("Invalid Property tag");
    curs.write_u32::<LittleEndian>(name.index).unwrap();
    write_u32(curs, self.name_variant);
    curs.write_u32::<LittleEndian>(tag.index).unwrap();
    write_u32(curs, self.tag_variant);
    curs.write_u64::<LittleEndian>(self.value.value_size() as u64).unwrap();
    if self.tag != "ArrayProperty" {
      // See note in self.read, this is bad
      curs.write(&[0]).unwrap();
    }
    self.value.write(curs, names, imports, exports);
  }

  pub fn byte_size(&self) -> usize {
    // 8 bytes each for name, tag, and size, 1 byte for random padding and obviously size bytes for the value
    25 + self.value.byte_size()
  }
}

impl Struct {
  pub fn read(rdr: &mut Cursor<Vec<u8>>, export: &ObjectExport, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> Result<Self, String> {
    println!("Export starting at {}[{:#04X}]", export.export_file_offset, export.export_file_offset);
    if rdr.position() != export.export_file_offset.into() {
      return Err(
        format!(
          "Error parsing Struct: Expected to be at position {:04X}, but I'm at position {:04X}",
          export.export_file_offset,
          rdr.position()
        )
        .to_string(),
      );
    }
    
    let start_pos = rdr.position();
    let mut properties = vec![];
    // Read properties until summary.tag is seen
    loop {
      let property = Property::read(rdr, names, imports, exports)?;
      match property {
        None => break,
        Some(prop) => {
          properties.push(prop);
        }
      }
    }
    let end_pos = rdr.position();
    // The length of all the properties read (including the None)
    let bytes_read = end_pos - start_pos;
    let remaining = export.serial_size - bytes_read;
    let extra = read_bytes(rdr, remaining as usize);
    Ok(Self { properties, extra })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> () {
    for prop in self.properties.iter() {
      prop.write(curs, names, imports, exports);
    }

    // Write none property
    let none = names
      .get_name_obj("None")
      .expect("None should be in names map");
    curs.write_u32::<LittleEndian>(none.index).unwrap();
    write_u32(curs, 0); // None name_variant
    write_u32(curs, 0);

    // Write extra data
    curs.write(&self.extra[..]).unwrap();
  }

  pub fn find(&mut self, name: &str) -> Option<&mut Property> {
    for prop in self.properties.iter_mut() {
      if prop.name == name {
        return Some(prop)
      }
    }
    return None
  }

  pub fn byte_size(&self) -> usize {
    // Size of properties plus size of None property at end of the struct and the extra data
    self.properties.iter().map(Property::byte_size).sum::<usize>() + 12 + self.extra.len()
  }

  pub fn total_size(structs: &Vec<Struct>) -> usize {
    structs.iter().map(Struct::byte_size).sum()
  }
}
