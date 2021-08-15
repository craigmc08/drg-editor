use crate::asset::*;
use crate::property::Property;
use crate::property::PropertyContext;
use crate::reader::*;
use crate::util::read_bytes;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::convert::TryInto;

pub static mut STRUCT_PATTERNS: Option<StructPatterns> = None;

#[derive(Debug)]
struct BinaryPropertyPattern {
  name: String,
  pattern: StructPattern,
}

#[derive(Debug)]
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
}

impl BinaryPropertyPattern {
  fn from_json(val: &JsonValue) -> Result<Self> {
    match &val["name"] {
      JsonValue::Null => bail!("Missing name property in binary property pattern"),
      JsonValue::String(name) => {
        let pattern =
          StructPattern::from_json(val).with_context(|| "In binary property pattern")?;
        Ok(Self {
          name: name.clone(),
          pattern,
        })
      }
      _ => bail!("Name property in binary property pattern is the wrong type (expected number)"),
    }
  }
}

impl StructPattern {
  /// Read numeric size from string like "i32", but in bytes instead of bits.
  //
  // # Examples
  //
  // ```
  // StructPattern::read_numeric_size("i32") == Ok(4)
  // ```
  fn read_numeric_size(typ: &str) -> Result<u8> {
    let size: u8 = typ[1..].parse()?;
    if size != 8 && size != 16 && size != 32 && size != 64 {
      bail!(
        "Invalid size for numeric pattern {}. Must be 8, 16, 32, or 64",
        size
      );
    } else {
      Ok(size / 8) // Size in bytes
    }
  }

  fn from_json(val: &JsonValue) -> Result<Self> {
    match val["type"].as_str() {
      None => bail!("Missing type property in struct pattern"),

      Some("property-list") => Ok(Self::PropertyList),

      Some("binary") => match &val["size"] {
        JsonValue::Null => bail!("Missing size property in struct pattern of type 'binary'"),
        JsonValue::Number(size) => Ok(Self::Binary {
          size: size.as_u64().unwrap() as usize,
        }),
        _ => bail!(
          "Size property in struct pattern of type 'binary' is the wrong type (expected number)"
        ),
      },

      Some("binary-properties") => match val["properties"].as_array() {
        None => bail!("Missing properties property in struct pattern of type 'binary-property'"),
        Some(props) => {
          let properties = props
            .iter()
            .map(BinaryPropertyPattern::from_json)
            .collect::<Result<Vec<BinaryPropertyPattern>>>()
            .with_context(|| "In binary-properties")?;
          Ok(Self::BinaryProperties { properties })
        }
      },

      Some(typ) => {
        if &typ[0..1] == "i" {
          Ok(Self::Int {
            size: Self::read_numeric_size(typ)?,
          })
        } else if &typ[0..1] == "u" {
          Ok(Self::UInt {
            size: Self::read_numeric_size(typ)?,
          })
        } else if &typ[0..1] == "f" {
          Ok(Self::Floating {
            size: Self::read_numeric_size(typ)?,
          })
        } else {
          bail!("Unknown type property value for struct pattern '{}'", typ)
        }
      }
    }
  }

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

#[derive(Debug)]
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
    }
  }
}

impl StructPatterns {
  pub fn from_json(val: &JsonValue) -> Result<Self> {
    let default = match &val["default"] {
      JsonValue::Null => bail!("Missing required default key in struct patterns json"),
      default_pattern_json => StructPattern::from_json(default_pattern_json)
        .with_context(|| "In default pattern for struct patterns")?,
    };
    let patterns = match &val["patterns"] {
      JsonValue::Object(obj) => {
        let mut patterns = HashMap::default();
        for (name, pattern_json) in obj.iter() {
          patterns.insert(
            name.clone(),
            StructPattern::from_json(pattern_json)
              .with_context(|| format!("For pattern '{}' in struct patterns", name))?,
          );
        }
        patterns
      }
      _ => bail!("Missing/wrong type for patterns key in struct patterns json"),
    };
    Ok(Self { default, patterns })
  }

  pub fn from_file(fp: &Path) -> Result<Self> {
    let contents = std::fs::read_to_string(fp)?;
    let json: JsonValue = serde_json::from_str(&contents)?;
    Self::from_json(&json)
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
    let pattern = match self.patterns.get(struct_type) {
      None => &self.default,
      Some(pattern) => pattern,
    };
    pattern.deserialize(rdr, ctx)
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
