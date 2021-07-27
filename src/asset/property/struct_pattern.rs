use crate::asset::*;
use crate::property::PropertyContext;
use crate::property::{PropType, Property, Value};
use crate::reader::*;
use crate::util::read_bytes;
use anyhow::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;

struct BinaryPropertyPattern {
  name: String,
  pattern: StructPattern,
}

enum StructPattern {
  PropertyList,
  Binary {
    size: usize,
  },
  BinaryProperties {
    properties: Vec<BinaryPropertyPattern>,
  },

  PropertyValue {
    prop_type: PropType,
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

      Some("builtin") => match val["tag"]
        .as_str()
        .and_then(|str| PropType::from_str(str).ok())
      {
        None => bail!("Invalid builtin tag property"),
        Some(prop_type) => Ok(Self::PropertyValue { prop_type }),
      },

      Some(typ) => bail!("Unknown type property value for struct pattern '{}'", typ),
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
      Self::PropertyValue { prop_type } => {
        let loader = Property::get_loader_for(*prop_type)?;
        let tag = if loader.simple {
          Tag::Simple(*prop_type)
        } else {
          loader.deserialize_tag(rdr, ctx)?
        };
        let value = loader.deserialize_value(rdr, &tag, rdr.remaining_bytes() as u64, ctx)?;
        Ok(StructValue::PropertyValue {
          prop_type: *prop_type,
          tag,
          value,
        })
      }
      Self::BinaryProperties { properties } => {
        let mut values = HashMap::default();
        for entry in properties {
          let value = entry
            .pattern
            .deserialize(rdr, ctx)
            .with_context(|| format!("In binary-properties.{}", entry.name))?;
          values.insert(entry.name.clone(), value);
        }
        Ok(StructValue::BinaryProperties { properties: values })
      }
    }
  }
}

pub struct StructPatterns {
  default: StructPattern,
  patterns: HashMap<String, StructPattern>,
}

pub enum StructValue {
  PropertyList {
    properties: Vec<Property>,
    has_none: bool,
  },
  Binary {
    bytes: Vec<u8>,
  },
  BinaryProperties {
    properties: HashMap<String, StructValue>,
  },
  PropertyValue {
    prop_type: PropType,
    tag: Tag,
    value: Value,
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
      Self::PropertyValue {
        prop_type,
        tag,
        value,
      } => {
        let loader = Property::get_loader_for(*prop_type)?;
        if let Tag::Simple(_) = tag {
          loader.serialize_value(curs, value, tag, ctx)?;
        } else {
          loader.serialize_tag(curs, tag, ctx)?;
          loader.serialize_value(curs, value, tag, ctx)?;
        }
        Ok(())
      }
      Self::BinaryProperties { properties } => {
        for (_key, value) in properties.iter() {
          value.serialize(curs, ctx)?;
        }
        Ok(())
      }
    }
  }
}

impl StructPatterns {
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
