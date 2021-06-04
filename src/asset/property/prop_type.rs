use crate::asset::property::context::*;
use crate::asset::*;
use crate::reader::*;
use anyhow::*;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumString)]
pub enum PropType {
  IntProperty,
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
}

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
