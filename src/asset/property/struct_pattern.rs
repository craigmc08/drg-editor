use crate::asset::*;
use crate::property::Property;
use crate::property::PropertyContext;
use crate::reader::*;
use crate::util::read_bytes;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryInto;

pub static mut STRUCT_PATTERNS: Option<StructPatterns> = None;

#[derive(Debug, Deserialize)]
struct BinaryPropertyPattern {
  name: String,
  #[serde(flatten)]
  pattern: StructPattern,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StructPattern {
  PropertyList,
  Binary {
    size: usize,
  },
  BinaryProperties {
    properties: Vec<BinaryPropertyPattern>,
  },

  Int {
    size: u8,
  },
  UInt {
    size: u8,
  },
  Floating {
    size: u8,
  },
  Enum {
    variants: Vec<String>,
  },
}

impl StructPattern {
  fn deserialize(&self, rdr: &mut ByteReader, ctx: PropertyContext) -> Result<StructValue> {
    match self {
      Self::PropertyList => {
        let mut properties = vec![];
        let mut has_none = false;
        let mut i = 0;
        'structloop: while !rdr.at_end() {
          let start_pos = rdr.position();
          let property = Property::deserialize(rdr, ctx)
            .with_context(|| format!("Struct property-list[{}] at {:#X}", i, start_pos))?;
          i += 1;
          if let Some(property) = property {
            properties.push(property);
          } else {
            has_none = true;
            break 'structloop;
          }
        }
        Ok(StructValue::PropertyList {
          properties,
          has_none,
        })
      }
      Self::Binary { size } => {
        let bytes: Vec<u8> = read_bytes(rdr, *size)?;
        Ok(StructValue::Binary { bytes })
      }
      Self::Int { size } => {
        let value: i64 = match size {
          1 => rdr.read_i8()?.into(),
          2 => rdr.read_i16::<LittleEndian>()?.into(),
          4 => rdr.read_i32::<LittleEndian>()?.into(),
          8 => rdr.read_i64::<LittleEndian>()?.into(),
          _ => bail!("Invalid size {} for Int pattern", size),
        };
        Ok(StructValue::Int { size: *size, value })
      }
      Self::UInt { size } => {
        let value: u64 = match size {
          1 => rdr.read_u8()?.into(),
          2 => rdr.read_u16::<LittleEndian>()?.into(),
          4 => rdr.read_u32::<LittleEndian>()?.into(),
          8 => rdr.read_u64::<LittleEndian>()?.into(),
          _ => bail!("Invalid size {} for UInt pattern", size),
        };
        Ok(StructValue::UInt { size: *size, value })
      }
      Self::Floating { size } => {
        let value: f64 = match size {
          4 => rdr.read_f32::<LittleEndian>()?.into(),
          8 => rdr.read_f64::<LittleEndian>()?.into(),
          _ => bail!("Invalid size {} for Floating pattern", size),
        };
        Ok(StructValue::Floating { size: *size, value })
      }
      Self::Enum { variants } => {
        let value = rdr.read_u8()?;
        Ok(StructValue::Enum {
          variants: variants.clone(),
          value,
        })
      }
      Self::BinaryProperties { properties } => {
        let mut entries = vec![];
        for entry in properties {
          let value = entry
            .pattern
            .deserialize(rdr, ctx)
            .with_context(|| format!("In binary-properties.{}", entry.name))?;
          entries.push((entry.name.clone(), value));
        }
        Ok(StructValue::BinaryProperties { entries })
      }
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct StructPatterns {
  default: StructPattern,
  patterns: HashMap<String, StructPattern>,
}

#[derive(Debug, Clone)]
pub enum StructValue {
  PropertyList {
    properties: Vec<Property>,
    has_none: bool,
  },
  Binary {
    bytes: Vec<u8>,
  },
  BinaryProperties {
    entries: Vec<(String, StructValue)>, // assoc. list to preserve ordering
  },

  Int {
    size: u8,
    value: i64,
  },
  UInt {
    size: u8,
    value: u64,
  },
  Floating {
    size: u8,
    value: f64,
  },
  Enum {
    variants: Vec<String>,
    value: u8,
  },
}

impl StructValue {
  pub fn serialize(&self, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
    match self {
      Self::PropertyList {
        properties,
        has_none,
      } => {
        for property in properties {
          property
            .serialize(curs, ctx)
            .with_context(|| "In struct property-list")?;
        }
        if *has_none {
          let none_name = NameVariant::new("None", 0, ctx.names);
          none_name
            .write(curs, ctx.names)
            .with_context(|| "Struct property-list none-terminator")?;
        }
        Ok(())
      }
      Self::Binary { bytes } => curs.write_all(bytes).with_context(|| "Struct binary data"),
      Self::BinaryProperties { entries } => {
        for (_key, value) in entries.iter() {
          value.serialize(curs, ctx)?;
        }
        Ok(())
      }
      Self::Int { size, value } => {
        match size {
          1 => curs.write_i8((*value).try_into()?)?,
          2 => curs.write_i16::<LittleEndian>((*value).try_into()?)?,
          4 => curs.write_i32::<LittleEndian>((*value).try_into()?)?,
          8 => curs.write_i64::<LittleEndian>((*value).try_into()?)?,
          _ => unreachable!(),
        }
        Ok(())
      }
      Self::UInt { size, value } => {
        match size {
          1 => curs.write_u8((*value).try_into()?)?,
          2 => curs.write_u16::<LittleEndian>((*value).try_into()?)?,
          4 => curs.write_u32::<LittleEndian>((*value).try_into()?)?,
          8 => curs.write_u64::<LittleEndian>((*value).try_into()?)?,
          _ => unreachable!(),
        }
        Ok(())
      }
      Self::Floating { size, value } => {
        match size {
          4 => curs.write_f32::<LittleEndian>(*value as f32)?,
          8 => curs.write_f64::<LittleEndian>(*value)?,
          _ => unreachable!(),
        }
        Ok(())
      }
      Self::Enum { value, .. } => {
        curs.write_u8(*value)?;
        Ok(())
      }
    }
  }

  pub fn byte_size(&self) -> usize {
    match self {
      Self::PropertyList {
        properties,
        has_none,
      } => {
        let none_size = if *has_none { 8 } else { 0 };
        properties
          .iter()
          .map(|prop| prop.byte_size())
          .sum::<usize>()
          + none_size
      }
      Self::Binary { bytes } => bytes.len(),
      Self::BinaryProperties { entries } => entries.iter().map(|(_, v)| v.byte_size()).sum(),
      Self::Int { size, .. } => *size as usize,
      Self::UInt { size, .. } => *size as usize,
      Self::Floating { size, .. } => *size as usize,
      Self::Enum { .. } => 1,
    }
  }
}

impl StructPatterns {
  pub fn from_file(fp: &Path) -> Result<Self> {
    let contents = std::fs::read_to_string(fp)?;
    let value = serde_json::from_str(&contents)?;
    Ok(value)
  }

  /// Loads struct patterns from fp into static instance, get it with `StructPatterns::get()`
  pub fn load(fp: &Path) -> Result<()> {
    let instance = Self::from_file(fp)?;
    unsafe {
      STRUCT_PATTERNS = Some(instance);
    }
    Ok(())
  }

  pub fn get() -> Option<&'static Self> {
    unsafe { STRUCT_PATTERNS.as_ref() }
  }

  pub fn deserialize(
    &self,
    rdr: &mut ByteReader,
    struct_type: &str,
    ctx: PropertyContext,
  ) -> Result<StructValue> {
    let (pattern, is_default) = match self.patterns.get(struct_type) {
      None => (&self.default, true),
      Some(pattern) => (pattern, false),
    };
    pattern
      .deserialize(rdr, ctx)
      .with_context(|| format!("For struct type {} (default = {})", struct_type, is_default))
  }

  pub fn serialize(
    &self,
    value: &StructValue,
    curs: &mut Cursor<Vec<u8>>,
    ctx: PropertyContext,
  ) -> Result<()> {
    value.serialize(curs, ctx)
  }
}
