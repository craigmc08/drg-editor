use crate::asset::property::context::*;
use crate::asset::*;
use crate::reader::*;
use anyhow::*;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
pub enum PropType {
  IntProperty,
  UInt8Property,
  FloatProperty,
  ObjectProperty,
  SoftObjectProperty,
  NameProperty,
  StrProperty,
  TextProperty,

  BoolProperty,
  ByteProperty,
  EnumProperty,
  ArrayProperty,
  StructProperty,
  MapProperty,
}

pub const ALL_PROP_TYPES: [PropType; 14] = [
  PropType::IntProperty,
  PropType::UInt8Property,
  PropType::FloatProperty,
  PropType::ObjectProperty,
  PropType::SoftObjectProperty,
  PropType::NameProperty,
  PropType::StrProperty,
  PropType::TextProperty,
  PropType::BoolProperty,
  PropType::ByteProperty,
  PropType::EnumProperty,
  PropType::ArrayProperty,
  PropType::StructProperty,
  PropType::MapProperty,
];

impl PropType {
  pub fn deserialize(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Self> {
    let name = NameVariant::read(rdr, ctx.names).with_context(|| "Deserializing PropType")?;
    let typ = PropType::from_str(&name.to_string(ctx.names))
      .with_context(|| format!("Parsing PropType {}", name.to_string(ctx.names)))?;
    Ok(typ)
  }

  pub fn serialize(&self, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
    let name: NameVariant = NameVariant::parse(&self.to_string(), ctx.names);
    name
      .write(curs, ctx.names)
      .with_context(|| "Serializing PropType")?;
    Ok(())
  }
}
