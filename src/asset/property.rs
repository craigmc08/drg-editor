use crate::asset::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;
use std::io::Cursor;
use std::convert::TryInto;

#[derive(Debug, Clone, Copy)]
pub enum PropertyTag {
  BoolProperty,
  ByteProperty,
  Int8Property,
  Int16Property,
  IntProperty,
  Int64Property,
  UInt16Property,
  UInt32Property,
  UInt64Property,
  FloatProperty,
  DoubleProperty,
  // EnumProperty, TODO
  // TextProperty, TODO
  // StrProperty, TODO
  // NameProperty, TODO
  ArrayProperty,
  // MapProperty, TODO
  ObjectProperty,
  StructProperty,
  // DebugProperty, TODO
  // SetProperty, TODO
  // WeakObjectProperty, TODO
  // LazyObjectProperty, TODO
  SoftObjectProperty,
  // DelegateProperty, TODO
  // MulticastDelegateProperty, TODO
  // InterfaceProperty, TODO
  // FieldPathProperty, TODO
  // AssetObjectProperty, TODO
}

impl PropertyTag {
  pub fn new(tag: &str) -> Result<Self, String> {
    match tag {
      "BoolProperty" => Ok(Self::BoolProperty),
      "ByteProperty" => Ok(Self::ByteProperty),
      "Int8Property" => Ok(Self::Int8Property),
      "Int16Property" => Ok(Self::Int16Property),
      "IntProperty" => Ok(Self::IntProperty),
      "Int64Property" => Ok(Self::Int64Property),
      "UInt16Property" => Ok(Self::UInt16Property),
      "UInt32Property" => Ok(Self::UInt32Property),
      "UInt64Property" => Ok(Self::UInt64Property),
      "FloatProperty" => Ok(Self::FloatProperty),
      "DoubleProperty" => Ok(Self::DoubleProperty),
      "ArrayProperty" => Ok(Self::ArrayProperty),
      "ObjectProperty" => Ok(Self::ObjectProperty),
      "StructProperty" => Ok(Self::StructProperty),
      "SoftObjectProperty" => Ok(Self::SoftObjectProperty),
      _ => Err(format!("Unimplemented tag type {}", tag))
    }
  }

  pub fn to_string(&self) -> &str {
    match self {
      Self::BoolProperty => "BoolProperty",
      Self::ByteProperty => "ByteProperty",
      Self::Int8Property => "Int8Property",
      Self::Int16Property => "Int16Property",
      Self::IntProperty => "IntProperty",
      Self::Int64Property => "Int64Property",
      Self::UInt16Property => "UInt16Property",
      Self::UInt32Property => "UInt32Property",
      Self::UInt64Property => "UInt64Property",
      Self::FloatProperty => "FloatProperty",
      Self::DoubleProperty => "DoubleProperty",
      Self::ArrayProperty => "ArrayProperty",
      Self::ObjectProperty => "ObjectProperty",
      Self::StructProperty => "StructProperty",
      Self::SoftObjectProperty => "SoftObjectProperty"
    }
  }

  pub fn read(rdr: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<Self, String> {
    let (tag, _) = names.read_name_with_variant(rdr, &format!("Property Tag @ #{:04X}", rdr.position()))?;
    Self::new(tag.as_ref())
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> () {
    names.write_name_with_variant(curs, self.to_string(), 0, "PropertyTag");
  }

  pub fn byte_size(&self) -> usize {
    8
  }
}

#[derive(Debug)]
pub enum PropertyTagData {
  EmptyTag { tag: PropertyTag },
  BoolTag { value: bool },
  ArrayTag { value_tag: PropertyTag },
  StructTag { name: String, name_variant: u32, guid: [u8; 16] },
}

impl PropertyTagData {
  // MUST be kept in sync with PropertyTag::is_complex_array_value
  pub fn read(tag: PropertyTag, rdr: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<Self, String> {
    let data = match tag {
      PropertyTag::BoolProperty => {
        Self::BoolTag { value: rdr.read_u8().unwrap() != 0 }
      }
      PropertyTag::ArrayProperty => {
        Self::ArrayTag { value_tag: PropertyTag::read(rdr, names)? }
      }
      PropertyTag::StructProperty => {
        let (name, name_variant) = names.read_name_with_variant(rdr, "StructTag.name")?;
        let guid = read_bytes(rdr, 16)[0..16].try_into().unwrap();
        Self::StructTag { name, name_variant, guid }
      }
      _ => Self::EmptyTag { tag }
    };
    rdr.consume(1); // eat the null-terminating byte after the tag data
    Ok(data)
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> () {
    match self {
      Self::EmptyTag { .. } => {}
      Self::BoolTag { value } => {
        curs.write_u8(if *value { 1 } else { 0 }).unwrap();
      }
      Self::ArrayTag { value_tag } => {
        value_tag.write(curs, names);
      }
      Self::StructTag { name, name_variant, guid } => {
        names.write_name_with_variant(curs, name, *name_variant, "StructTag.name");
      }
    }
    curs.write(&[0]); // write the null-terminating byte
  }

  // Includes the null-terminating byte
  pub fn byte_size(&self) -> usize {
    match self {
      Self::EmptyTag { .. } => 1, // just padding
      Self::BoolTag { .. } => 2, // u8 value + padding
      Self::ArrayTag { value_tag } => value_tag.byte_size() + 1, // tag size + padding
      Self::StructTag { .. } => 8 + 16 + 1, // name + guid[u8; 16] + padding
    }
  }
}

#[derive(Debug)]
pub enum ArrayValue {
  Simple { value: PropertyValue },
  Complex { value: Property },
}

impl PropertyTag {
  // This MUST be kept in-sync with PropertyTagData::read/write
  pub fn is_complex_array_value(&self) -> bool {
    // As far as I can tell, an array has a Complex value when it's value has
    // a non-Empty PropertyTagData
    match self {
      Self::BoolProperty => true,
      Self::ArrayProperty => true,
      Self::StructProperty => true,
      _ => false
    }
  }
}

impl ArrayValue {
  pub fn read(tag: &PropertyTag, rdr: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> Result<Self, String> {
    if tag.is_complex_array_value() {
      Ok(Self::Complex { value: Property::read(rdr, names, imports, exports)?.unwrap() })
    } else {
      let tag_data = PropertyTagData::EmptyTag { tag: *tag };
      // Size only matters for Complex types, at least for now
      Ok(Self::Simple { value: PropertyValue::read(rdr, 0, &tag_data, names, imports, exports)? })
    }
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> () {
    match self {
      Self::Complex { value } => value.write(curs, names, imports, exports),
      Self::Simple { value } => value.write(curs, names, imports, exports)
    }
  }

  pub fn byte_size(&self) -> usize {
    match self {
      Self::Simple { value } => value.byte_size(),
      Self::Complex { value } => value.byte_size(),
    }
  }
}

#[derive(Debug)]
pub enum PropertyValue {
  BoolProperty { },
  ByteProperty {
    value: u8,
  },
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
  ArrayProperty {
    values: Vec<ArrayValue>,
  },
  ObjectProperty {
    value: Dependency,
  },
  StructProperty {
    data: Vec<u8>
  },
  SoftObjectProperty {
    object_name: String,
    object_name_variant: u32,
    unk1: u32,
  },
}

impl PropertyValue {
  fn read(
    rdr: &mut Cursor<Vec<u8>>,
    size: u64,
    tag_data: &PropertyTagData,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<Self, String> {
    match tag_data {
      PropertyTagData::EmptyTag { tag: PropertyTag::ByteProperty } => Ok(Self::ByteProperty {
        value: rdr.read_u8().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::Int8Property } => Ok(Self::Int8Property {
        value: rdr.read_i8().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::Int16Property } => Ok(Self::Int16Property {
        value: rdr.read_i16::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::IntProperty } => Ok(Self::IntProperty {
        value: rdr.read_i32::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::Int64Property } => Ok(Self::Int64Property {
        value: rdr.read_i64::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::UInt16Property } => Ok(Self::UInt16Property {
        value: rdr.read_u16::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::UInt32Property } => Ok(Self::UInt32Property {
        value: read_u32(rdr),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::UInt64Property } => Ok(Self::UInt64Property {
        value: rdr.read_u64::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::FloatProperty } => Ok(Self::FloatProperty {
        value: rdr.read_f32::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::DoubleProperty } => Ok(Self::DoubleProperty {
        value: rdr.read_f64::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag { tag: PropertyTag::ObjectProperty } => {
        let value = Dependency::read(rdr, imports, exports)?;
        Ok(Self::ObjectProperty { value })
      }
      PropertyTagData::EmptyTag { tag: PropertyTag::SoftObjectProperty } => {
        let (object_name, object_name_variant) = names.read_name_with_variant(rdr, "SoftObjectProperty object_name")?;
        let unk1 = read_u32(rdr);
        Ok(Self::SoftObjectProperty { object_name, object_name_variant, unk1 })
      }

      PropertyTagData::EmptyTag { tag } => Err(format!("Illegal PropertyTagData object {:?}, report to maintainer", tag)),
      
      PropertyTagData::BoolTag { .. } => Ok(Self::BoolProperty {}),
      PropertyTagData::ArrayTag { value_tag } => {
        let length = read_u32(rdr);
        let mut values = vec![];
        for _ in 0..length {
          let value = ArrayValue::read(value_tag, rdr, names, imports, exports)?;
          values.push(value);
        }
        Ok(Self::ArrayProperty { values })
      }
      PropertyTagData::StructTag { .. } => {
        let data = read_bytes(rdr, size as usize);
        Ok(Self::StructProperty { data })
      }
    }
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> () {
    match self {
      Self::ByteProperty { value } => curs.write_u8(*value).unwrap(),
      Self::Int8Property { value } => curs.write_i8(*value).unwrap(),
      Self::Int16Property { value } => curs.write_i16::<LittleEndian>(*value).unwrap(),
      Self::IntProperty { value } => curs.write_i32::<LittleEndian>(*value).unwrap(),
      Self::Int64Property { value } => curs.write_i64::<LittleEndian>(*value).unwrap(),
      Self::UInt16Property { value } => curs.write_u16::<LittleEndian>(*value).unwrap(),
      Self::UInt32Property { value } => curs.write_u32::<LittleEndian>(*value).unwrap(),
      Self::UInt64Property { value } => curs.write_u64::<LittleEndian>(*value).unwrap(),
      Self::FloatProperty { value } => curs.write_f32::<LittleEndian>(*value).unwrap(),
      Self::DoubleProperty { value } => curs.write_f64::<LittleEndian>(*value).unwrap(),
      Self::ObjectProperty { value } => {
        value.write(curs, imports, exports);
      }
      Self::SoftObjectProperty { object_name, object_name_variant, unk1 } => {
        let object_name_n = names
          .get_name_obj(object_name)
          .expect("Invalid SoftObjectProperty object_name");
        curs.write_u32::<LittleEndian>(object_name_n.index).unwrap();
        write_u32(curs, *object_name_variant);
        write_u32(curs, *unk1);
      }
      Self::BoolProperty { } => { },
      Self::ArrayProperty { values } => {
        write_u32(curs, values.len() as u32);
        for value in values.iter() {
          value.write(curs, names, imports, exports);
        }
      }
      Self::StructProperty { data } => {
        curs.write(&data[..]).unwrap();
      }
    }
  }

  pub fn byte_size(&self) -> usize {
    match self {
      Self::BoolProperty { } => 0,
      Self::ByteProperty { .. } => 1,
      Self::Int8Property { .. } => 1,
      Self::Int16Property { .. } => 2,
      Self::IntProperty { .. } => 4,
      Self::Int64Property { .. } => 8,
      Self::UInt16Property { .. } => 2,
      Self::UInt32Property { .. } => 4,
      Self::UInt64Property { .. } => 8,
      Self::FloatProperty { .. } => 4,
      Self::DoubleProperty { .. } => 8,
      Self::ArrayProperty { values, .. } => {
        // u32 size + values
        4 + values.into_iter().map(|x| x.byte_size()).sum::<usize>()
      }
      Self::ObjectProperty { .. } => 4,
      Self::StructProperty { data } => data.len(),
      Self::SoftObjectProperty { .. } => 12,
    }
  }
}

#[derive(Debug)]
pub struct Property {
  pub name: String, // u632 index into name_map
  pub name_variant: u32,
  pub tag: PropertyTag,
  pub size: u64,
  pub tag_data: PropertyTagData,
  // 1 byte of padding?
  pub value: PropertyValue,
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

    let tag = PropertyTag::read(rdr, names)?;
    let size = rdr.read_u64::<LittleEndian>().unwrap();

    // Consume the byte of padding if the property type doesn't have it after some metadata
    // TODO:
    // The byte is a null-terminator after a list of properties for the tag
    // So consume all of those names and pass them to PropertyValue::read along with tag

    let tag_data = PropertyTagData::read(tag, rdr, names)?;

    let value = PropertyValue::read(rdr, size, &tag_data, names, imports, exports)?;
    // TODO: this can be uncommented after fixing the "padding byte"
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
      size,
      tag_data,
      value,
    }));
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap, imports: &ObjectImports, exports: &ObjectExports) -> () {
    names.write_name_with_variant(curs, &self.name, self.name_variant, "Property.name");
    self.tag.write(curs, names);
    curs.write_u64::<LittleEndian>(self.size).unwrap();
    self.tag_data.write(curs, names);
    self.value.write(curs, names, imports, exports);
  }

  pub fn byte_size(&self) -> usize {
    // 8 bytes each for name, tag, and size, 1 byte for random padding and obviously size bytes for the value
    25 + self.value.byte_size()
  }
}

#[derive(Debug)]
pub struct Struct {
  pub properties: Vec<Property>,
  pub extra: Vec<u8>, // extra unknown info after serial_size for export property
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
