use crate::asset::*;
use crate::property::PropertyContext;
use crate::property::{PropType, Property, Value};
use crate::reader::*;
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
    tag: PropType,
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
        Some(tag) => Ok(Self::PropertyValue { tag }),
      },

      Some(typ) => bail!("Unknown type property value for struct pattern '{}'", typ),
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
  },
  Binary {
    bytes: Vec<u8>,
  },
  BinaryProperties {
    properties: HashMap<String, StructValue>,
  },
  PropertyValue {
    value: Value,
  },
}

impl StructPatterns {
  pub fn deserialize(&self, rdr: &mut ByteReader, ctx: PropertyContext) -> Result<StructValue> {
    panic!("Unimplemented")
  }

  pub fn serialize(
    &self,
    value: &StructValue,
    curs: &mut Cursor<Vec<u8>>,
    ctx: PropertyContext,
  ) -> Result<()> {
    panic!("Unimplemented")
  }
}
