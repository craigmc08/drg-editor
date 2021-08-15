use crate::asset::struct_pattern::*;
use crate::asset::*;
use crate::reader::*;
use crate::util::*;
use anyhow::*;
pub use context::PropertyContext;
use std::io::prelude::*;

mod context;
mod loaders;
pub mod meta;
pub mod prop_type;
pub mod struct_pattern;

use loaders::{PropertyLoader, LOADERS};
use meta::*;
use prop_type::*;

/*====================
// Property Components
====================*/

#[derive(Debug, Clone)]
pub enum Value {
  Int(i32),
  Float(f32),
  Object(Reference),
  // TODO: unk1 might be the length of a string that is stored in the SoftObjectProperty
  // Find example of this happening?
  SoftObject {
    object_name: NameVariant,
    unk1: u32,
  },
  Name(NameVariant),
  Str(String),
  Text {
    header: [u8; 9],
    bytes: Vec<u8>,
    text: String,
  },

  Bool,
  Enum(NameVariant), // For ByteProperty and EnumProperty, TODO better name?
  Array {
    values: Vec<Value>,
  },
  Struct {
    value: StructValue,
  },
  RawData {
    data: Vec<u8>,
  },
  // etc.
}

#[derive(Debug, Clone)]
pub enum Tag {
  Simple(PropType),
  Bool(bool),
  Enum(NameVariant), // For ByteProperty and EnumProperty, TODO better name?
  Array {
    inner_type: PropType,
  },
  Struct {
    type_name: NameVariant,
    guid: [u8; 16],
  }, // Etc.
}

#[derive(Debug, Clone)]
pub struct Property {
  pub meta: Meta,
  pub tag: Tag,
  pub value: Value,
}

impl Property {
  pub fn get_loader_for<'a>(typ: PropType) -> Result<&'a PropertyLoader<'a>> {
    LOADERS
      .iter()
      .find(|l| l.is_for_type(typ))
      .ok_or_else(|| anyhow!("No reader for {}", typ))
  }

  pub fn deserialize(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Option<Self>> {
    if let Some(meta) = Meta::deserialize(rdr, ctx)? {
      let loader = Self::get_loader_for(meta.typ)?;

      // println!("Entering tag for {} at {:#X}", meta.typ, rdr.position());
      let tag = loader.deserialize_tag(rdr, ctx)?;

      // Read 0x00 between tag and value
      rdr.read_exact(&mut [0])?;

      // println!("Entering value for {} at {:#X}", meta.typ, rdr.position());

      rdr.limit(meta.size as usize);
      let value = loader.deserialize_value(rdr, &tag, meta.size, ctx)?;
      rdr.unlimit();

      // println!("Exiting value for {} at {:#X} \n", meta.typ, rdr.position());

      Ok(Some(Self { meta, tag, value }))
    } else {
      Ok(None)
    }
  }

  pub fn serialize(&self, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
    let loader = Self::get_loader_for(self.meta.typ)?;

    // Compute real size
    let new_size = self.byte_size() - self.meta.byte_size();
    let meta = Meta::new(self.meta.name.clone(), self.meta.typ, new_size as u64);
    meta.serialize(curs, ctx)?;

    loader.serialize_tag(curs, &self.tag, ctx)?;
    curs.write_all(&[0])?;
    loader.serialize_value(curs, &self.value, &self.tag, ctx)?;
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    let loader = Self::get_loader_for(self.meta.typ).expect("Expected valid type");
    let meta_size = self.meta.byte_size();
    let tag_size = loader.tag_size(&self.tag);
    let value_size = loader.value_size(&self.value, &self.tag);
    // Include 0x00 byte separator between tag and value
    meta_size + tag_size + 1 + value_size
  }
}

// Properties list serialization stuff
pub struct Properties {
  pub properties: Vec<Property>,
  ends_with_none: bool,
  extra: Vec<u8>,
}

impl Properties {
  pub fn deserialize(rdr: &mut ByteReader, export: &Export, ctx: PropertyContext) -> Result<Self> {
    // Check that start position is correct
    let start_pos = rdr.position();
    if start_pos != export.export_file_offset {
      bail!(
        "Wrong properties starting position for {}: Expected to be at position {:#X}, but I'm at position {:#X}",
        export.object_name.to_string(ctx.names), export.export_file_offset, start_pos
      );
    }
    let mut ends_with_none = false;
    let mut properties = vec![];

    // Read all properties until None prop or past end of this export
    let end_pos = export.export_file_offset + export.serial_size;
    'proploop: while rdr.position() < end_pos {
      let start_pos = rdr.position();

      if let Some(prop) = Property::deserialize(rdr, ctx).with_context(|| {
        format!(
          "Property in {} starting at {:#X}",
          export.object_name.to_string(ctx.names),
          start_pos
        )
      })? {
        properties.push(prop);
      } else {
        ends_with_none = true;
        break 'proploop;
      }
    }

    // Check that not too many bytes were read
    let num_bytes_read = rdr.position() - start_pos;
    if num_bytes_read > export.serial_size {
      bail!(
        "Properties length for {} too long: Expected to read at most {:#X} bytes, but I read {:#X}",
        export.object_name.to_string(ctx.names),
        export.serial_size,
        num_bytes_read
      );
    }

    // Read any bytes that were left over. This happens sometimes, and I'm not sure why.
    let extra = if num_bytes_read < export.serial_size {
      let remaining = export.serial_size as usize - num_bytes_read as usize;
      read_bytes(rdr, remaining).with_context(|| {
        format!(
          "Properties extra data, filling {} - {}/{}",
          remaining, num_bytes_read, export.serial_size
        )
      })?
    } else {
      vec![]
    };

    Ok(Self {
      properties,
      ends_with_none,
      extra,
    })
  }

  pub fn serialize(&self, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
    for property in &self.properties {
      property
        .serialize(curs, ctx)
        .with_context(|| "While serializing struct")?;
    }

    if self.ends_with_none {
      let none: NameVariant = NameVariant::new("None", 0, ctx.names);
      none
        .write(curs, ctx.names)
        .with_context(|| "Expected None in names")?;
    }

    curs.write_all(&self.extra[..])?;

    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    let props_size = self.properties.iter().map(|p| p.byte_size()).sum::<usize>();
    let none_size = if self.ends_with_none { 8 } else { 0 };
    let extra_size = self.extra.len();
    props_size + none_size + extra_size
  }
}
