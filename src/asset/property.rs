use crate::asset::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryInto;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::{Seek, SeekFrom};

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
  TextProperty,
  // StrProperty, TODO
  NameProperty,

  EnumProperty,
  ArrayProperty,
  MapProperty,
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
      "TextProperty" => Ok(Self::TextProperty),
      "NameProperty" => Ok(Self::NameProperty),
      "EnumProperty" => Ok(Self::EnumProperty),
      "ArrayProperty" => Ok(Self::ArrayProperty),
      "MapProperty" => Ok(Self::MapProperty),
      "ObjectProperty" => Ok(Self::ObjectProperty),
      "StructProperty" => Ok(Self::StructProperty),
      "SoftObjectProperty" => Ok(Self::SoftObjectProperty),
      _ => Err(format!("Unimplemented tag type {}", tag)),
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
      Self::TextProperty => "TextProperty",
      Self::NameProperty => "NameProperty",
      Self::EnumProperty => "EnumProperty",
      Self::ArrayProperty => "ArrayProperty",
      Self::MapProperty => "MapProperty",
      Self::ObjectProperty => "ObjectProperty",
      Self::StructProperty => "StructProperty",
      Self::SoftObjectProperty => "SoftObjectProperty",
    }
  }

  pub fn read(rdr: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<Self, String> {
    let (tag, _) =
      names.read_name_with_variant(rdr, &format!("Property Tag @ #{:04X}", rdr.position()))?;
    Self::new(tag.as_ref())
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> () {
    names.write_name_with_variant(curs, self.to_string(), 0, "PropertyTag");
  }

  pub fn byte_size(&self) -> usize {
    8
  }
}

#[derive(Debug, Clone)]
pub enum PropertyTagData {
  EmptyTag {
    tag: PropertyTag,
  },
  BoolTag {
    value: bool,
  },
  EnumTag {
    name: String,
    name_variant: u32,
  },
  ArrayTag {
    value_tag: PropertyTag,
  },
  MapTag {
    key_tag: PropertyTag,
    value_tag: PropertyTag,
  },
  StructTag {
    name: String,
    name_variant: u32,
    guid: [u8; 16],
  },
}

impl PropertyTagData {
  // MUST be kept in sync with PropertyTag::is_complex_array_value
  pub fn read(
    tag: PropertyTag,
    rdr: &mut Cursor<Vec<u8>>,
    names: &NameMap,
  ) -> Result<Self, String> {
    let data = match tag {
      PropertyTag::BoolProperty => Self::BoolTag {
        value: rdr.read_u8().unwrap() != 0,
      },
      PropertyTag::EnumProperty => {
        let (name, name_variant) = names.read_name_with_variant(rdr, "EnumTag.name")?;
        Self::EnumTag { name, name_variant }
      }
      PropertyTag::ArrayProperty => Self::ArrayTag {
        value_tag: PropertyTag::read(rdr, names)?,
      },
      PropertyTag::MapProperty => {
        let key_tag = PropertyTag::read(rdr, names)?;
        let value_tag = PropertyTag::read(rdr, names)?;
        Self::MapTag { key_tag, value_tag }
      }
      PropertyTag::StructProperty => {
        let (name, name_variant) = names.read_name_with_variant(rdr, "StructTag.name")?;
        let guid = read_bytes(rdr, 16)[0..16].try_into().unwrap();
        Self::StructTag {
          name,
          name_variant,
          guid,
        }
      }
      _ => Self::EmptyTag { tag },
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
      Self::EnumTag { name, name_variant } => {
        names.write_name_with_variant(curs, name, *name_variant, "EnumTag.name");
      }
      Self::ArrayTag { value_tag } => {
        value_tag.write(curs, names);
      }
      Self::MapTag { key_tag, value_tag } => {
        key_tag.write(curs, names);
        value_tag.write(curs, names);
      }
      Self::StructTag {
        name,
        name_variant,
        guid,
      } => {
        names.write_name_with_variant(curs, name, *name_variant, "StructTag.name");
        curs.write(guid).unwrap();
      }
    }
    curs.write(&[0]).unwrap(); // write the null-terminating byte
  }

  // Includes the null-terminating byte
  pub fn byte_size(&self) -> usize {
    let data_size = match self {
      Self::EmptyTag { .. } => 0,
      Self::BoolTag { .. } => 1,                             // a single u8
      Self::EnumTag { .. } => 8,                             // name + name_variant
      Self::ArrayTag { value_tag } => value_tag.byte_size(), // tag size
      Self::MapTag { key_tag, value_tag } => key_tag.byte_size() + value_tag.byte_size(), // tags size
      Self::StructTag { .. } => 8 + 16, // name + name_variant + guid[u8; 16]
    };
    data_size + 1
  }
}

#[derive(Debug, Clone)]
pub enum NestedValue {
  Simple { value: PropertyValue },
  Complex { value: Option<Property> },
}

impl PropertyTag {
  // This MUST be kept in-sync with PropertyTagData::read/write
  pub fn is_complex_array_value(&self) -> bool {
    // As far as I can tell, an array has a Complex value when it's value has
    // a non-Empty PropertyTagData
    match self {
      Self::BoolProperty => true,
      Self::ArrayProperty => true,
      Self::MapProperty => true,
      Self::StructProperty => true,
      _ => false,
    }
  }
}

impl NestedValue {
  pub fn read(
    tag: &PropertyTag,
    rdr: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<Self, String> {
    if tag.is_complex_array_value() {
      let start_pos = rdr.position();
      match Property::read(rdr, names, imports, exports)? {
        None => {
          // If a None property was read, theen move back to before trying to read a property
          rdr.seek(SeekFrom::Start(start_pos)).unwrap();
          Ok(Self::Complex { value: None })
        }
        Some(prop) => Ok(Self::Complex { value: Some(prop) }),
      }
    } else {
      let tag_data = PropertyTagData::EmptyTag { tag: *tag };
      // Size only matters for Complex types, at least for now
      Ok(Self::Simple {
        value: PropertyValue::read(rdr, 0, &tag_data, names, imports, exports)?,
      })
    }
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> () {
    match self {
      Self::Complex { value: None } => {}
      Self::Complex { value: Some(prop) } => prop.write(curs, names, imports, exports),
      Self::Simple { value } => value.write(curs, names, imports, exports),
    }
  }

  pub fn byte_size(&self) -> usize {
    match self {
      Self::Simple { value } => value.byte_size(),
      Self::Complex { value } => value.as_ref().map(Property::byte_size).unwrap_or(0),
    }
  }
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
  BoolProperty {},
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
  TextProperty {
    bytes: [u8; 13], // TODO this might be wrong
    value: String,
  },
  NameProperty {
    name: String,
    name_variant: u32,
  },
  EnumProperty {
    value: String,
    value_variant: u32,
  },
  ArrayProperty {
    values: Vec<NestedValue>,
  },
  MapProperty {
    flags: u32,
    entries: Vec<(NestedValue, NestedValue)>,
  },
  ObjectProperty {
    value: Dependency,
  },
  StructProperty {
    data: Vec<u8>,
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
      PropertyTagData::EmptyTag {
        tag: PropertyTag::ByteProperty,
      } => Ok(Self::ByteProperty {
        value: rdr.read_u8().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::Int8Property,
      } => Ok(Self::Int8Property {
        value: rdr.read_i8().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::Int16Property,
      } => Ok(Self::Int16Property {
        value: rdr.read_i16::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::IntProperty,
      } => Ok(Self::IntProperty {
        value: rdr.read_i32::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::Int64Property,
      } => Ok(Self::Int64Property {
        value: rdr.read_i64::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::UInt16Property,
      } => Ok(Self::UInt16Property {
        value: rdr.read_u16::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::UInt32Property,
      } => Ok(Self::UInt32Property {
        value: read_u32(rdr),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::UInt64Property,
      } => Ok(Self::UInt64Property {
        value: rdr.read_u64::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::FloatProperty,
      } => Ok(Self::FloatProperty {
        value: rdr.read_f32::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::DoubleProperty,
      } => Ok(Self::DoubleProperty {
        value: rdr.read_f64::<LittleEndian>().unwrap(),
      }),
      PropertyTagData::EmptyTag {
        tag: PropertyTag::TextProperty,
      } => {
        let bytes = read_bytes(rdr, 13); // TODO
        let value = read_string(rdr);
        Ok(Self::TextProperty {
          bytes: bytes[..].try_into().unwrap(),
          value,
        })
      }
      PropertyTagData::EmptyTag {
        tag: PropertyTag::NameProperty,
      } => {
        let (name, name_variant) = names.read_name_with_variant(rdr, "NameProperty.name")?;
        Ok(Self::NameProperty { name, name_variant })
      }
      PropertyTagData::EmptyTag {
        tag: PropertyTag::ObjectProperty,
      } => {
        let value = Dependency::read(rdr, imports, exports)?;
        Ok(Self::ObjectProperty { value })
      }
      PropertyTagData::EmptyTag {
        tag: PropertyTag::SoftObjectProperty,
      } => {
        let (object_name, object_name_variant) =
          names.read_name_with_variant(rdr, "SoftObjectProperty object_name")?;
        let unk1 = read_u32(rdr);
        Ok(Self::SoftObjectProperty {
          object_name,
          object_name_variant,
          unk1,
        })
      }

      PropertyTagData::EmptyTag { tag } => Err(format!(
        "Illegal PropertyTagData object {:?}, report to maintainer",
        tag
      )),
      PropertyTagData::BoolTag { .. } => Ok(Self::BoolProperty {}),
      PropertyTagData::EnumTag { .. } => {
        let (value, value_variant) = names.read_name_with_variant(rdr, "EnumProperty.value")?;
        Ok(Self::EnumProperty {
          value,
          value_variant,
        })
      }
      PropertyTagData::ArrayTag { value_tag } => {
        let length = read_u32(rdr);
        let mut values = vec![];
        for _ in 0..length {
          let value = NestedValue::read(value_tag, rdr, names, imports, exports)?;
          values.push(value);
        }
        Ok(Self::ArrayProperty { values })
      }
      PropertyTagData::MapTag { key_tag, value_tag } => {
        let flags = read_u32(rdr);
        let num_entries = read_u32(rdr);
        let mut entries = vec![];
        for _ in 0..num_entries {
          let key = NestedValue::read(key_tag, rdr, names, imports, exports)?;
          let value = NestedValue::read(value_tag, rdr, names, imports, exports)?;
          entries.push((key, value));
        }
        Ok(Self::MapProperty { flags, entries })
      }
      PropertyTagData::StructTag { .. } => {
        let data = read_bytes(rdr, size as usize);
        Ok(Self::StructProperty { data })
      }
    }
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> () {
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
      Self::TextProperty { bytes, value } => {
        curs.write(bytes).unwrap();
        write_string(curs, value);
      }
      Self::NameProperty { name, name_variant } => {
        names.write_name_with_variant(curs, name, *name_variant, "NameProperty.name");
      }
      Self::ObjectProperty { value } => {
        value.write(curs, imports, exports);
      }
      Self::SoftObjectProperty {
        object_name,
        object_name_variant,
        unk1,
      } => {
        let object_name_n = names
          .get_name_obj(object_name)
          .expect("Invalid SoftObjectProperty object_name");
        curs.write_u32::<LittleEndian>(object_name_n.index).unwrap();
        write_u32(curs, *object_name_variant);
        write_u32(curs, *unk1);
      }
      Self::BoolProperty {} => {}
      Self::EnumProperty {
        value,
        value_variant,
      } => {
        names.write_name_with_variant(curs, value, *value_variant, "EnumProperty.value");
      }
      Self::ArrayProperty { values } => {
        write_u32(curs, values.len() as u32);
        for value in values.iter() {
          value.write(curs, names, imports, exports);
        }
      }
      Self::MapProperty { flags, entries } => {
        write_u32(curs, *flags);
        write_u32(curs, entries.len() as u32);
        for (key, value) in entries.iter() {
          key.write(curs, names, imports, exports);
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
      Self::BoolProperty {} => 0,
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
      Self::EnumProperty { .. } => 8,
      Self::TextProperty { bytes, value } => {
        // bytes + length + string value + null terminator
        bytes.len() + 4 + value.len() + 1
      }
      Self::NameProperty { .. } => 8,
      Self::ArrayProperty { values, .. } => {
        // u32 size + values
        4 + values.iter().map(|x| x.byte_size()).sum::<usize>()
      }
      Self::MapProperty { entries, .. } => {
        // flags + length + entries size
        4 + 4
          + entries
            .iter()
            .map(|(k, v)| k.byte_size() + v.byte_size())
            .sum::<usize>()
      }
      Self::ObjectProperty { .. } => 4,
      Self::StructProperty { data } => data.len(),
      Self::SoftObjectProperty { .. } => 12,
    }
  }
}

#[derive(Debug, Clone)]
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

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> () {
    names.write_name_with_variant(curs, &self.name, self.name_variant, "Property.name");
    self.tag.write(curs, names);
    curs.write_u64::<LittleEndian>(self.size).unwrap();
    self.tag_data.write(curs, names);
    self.value.write(curs, names, imports, exports);
  }

  pub fn byte_size(&self) -> usize {
    // 8 bytes for name, then tag size, then u64 size, then size of tag_data and value
    8 + self.tag.byte_size() + 8 + self.tag_data.byte_size() + self.value.byte_size()
  }
}

#[derive(Debug)]
pub struct Struct {
  pub properties: Vec<Property>,
  pub extra: Vec<u8>, // extra unknown info after serial_size for export property
}

impl Struct {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    export: &ObjectExport,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<Self, String> {
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

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> () {
    for prop in self.properties.iter() {
      prop.write(curs, names, imports, exports);
    }

    // Write none property
    let none = names
      .get_name_obj("None")
      .expect("None should be in names map");
    curs.write_u32::<LittleEndian>(none.index).unwrap();
    write_u32(curs, 0); // None name_variant

    // Write extra data
    curs.write(&self.extra[..]).unwrap();
  }

  pub fn find(&mut self, name: &str) -> Option<&mut Property> {
    for prop in self.properties.iter_mut() {
      if prop.name == name {
        return Some(prop);
      }
    }
    return None;
  }

  pub fn byte_size(&self) -> usize {
    // Size of properties plus size of None property at end of the struct and the extra data
    self
      .properties
      .iter()
      .map(Property::byte_size)
      .sum::<usize>()
      + 8
      + self.extra.len()
  }

  pub fn total_size(structs: &Vec<Struct>) -> usize {
    structs.iter().map(Struct::byte_size).sum()
  }
}
