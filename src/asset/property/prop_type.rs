use crate::asset::property::context::*;
use crate::asset::*;
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

  BoolProperty,
  ByteProperty,
  EnumProperty,
  ArrayProperty,
  StructProperty,
}

impl PropType {
  pub fn deserialize(rdr: Curs, ctx: PropertyContext) -> Result<Self> {
    let name = NameVariant::read(rdr, ctx.names).with_context(|| "Deserializing PropType")?;
    let typ = PropType::from_str(&name.to_string())
      .with_context(|| format!("Parsing PropType {}", name))?;
    Ok(typ)
  }

  pub fn serialize(&self, curs: Curs, ctx: PropertyContext) -> Result<()> {
    let name: NameVariant = self.to_string().into();
    name
      .write(curs, ctx.names)
      .with_context(|| "Serializing PropType")?;
    Ok(())
  }
}
