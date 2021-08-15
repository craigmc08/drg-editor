use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader;
use crate::reader::*;
use byteorder::{ReadBytesExt, WriteBytesExt};

pub const LOADER_BOOL: PropertyLoader = loader!(
  [PropType::BoolProperty],
  deserialize_bool_value,
  deserialize_bool_tag,
  |_, _, _, _| Ok(()),
  serialize_bool_tag,
  |_, _| 0,
  |_| 1,
);
pub const LOADER_ENUM: PropertyLoader = loader!(
  [PropType::EnumProperty, PropType::ByteProperty],
  deserialize_enum_value,
  deserialize_enum_tag,
  serialize_enum_value,
  serialize_enum_tag,
  |_, _| 8,
  |_| 8,
);

fn deserialize_bool_value(
  _: &mut ByteReader,
  _: &Tag,
  _: u64,
  _: PropertyContext,
) -> Result<Value> {
  Ok(Value::Bool)
}
fn deserialize_bool_tag(rdr: &mut ByteReader, _ctx: PropertyContext) -> Result<Tag> {
  let value = rdr.read_u8()? != 0;
  Ok(Tag::Bool(value))
}
/// # Panics
/// If tag is not Bool variant.
fn serialize_bool_tag(tag: &Tag, curs: &mut Cursor<Vec<u8>>, _ctx: PropertyContext) -> Result<()> {
  if let Tag::Bool(value) = tag {
    let value = if *value { 1 } else { 0 };
    curs.write_u8(value)?;
    Ok(())
  } else {
    unreachable!()
  }
}

fn deserialize_enum_value(
  rdr: &mut ByteReader,
  _: &Tag,
  _: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  Ok(Value::Enum(
    NameVariant::read(rdr, ctx.names).with_context(|| "Enum/Byte.value")?,
  ))
}
fn deserialize_enum_tag(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Tag> {
  Ok(Tag::Enum(
    NameVariant::read(rdr, ctx.names).with_context(|| "Enum/Byte.tag")?,
  ))
}
/// # Panics
/// If val is not Enum variant.
fn serialize_enum_value(
  val: &Value,
  _: &Tag,
  curs: &mut Cursor<Vec<u8>>,
  ctx: PropertyContext,
) -> Result<()> {
  if let Value::Enum(val) = val {
    val
      .write(curs, ctx.names)
      .with_context(|| "Enum/Byte.value")?;
    Ok(())
  } else {
    unreachable!()
  }
}
/// # Panics
/// If tag is not Enum variant.
fn serialize_enum_tag(tag: &Tag, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
  if let Tag::Enum(typ) = tag {
    typ
      .write(curs, ctx.names)
      .with_context(|| "Enum/Byte.tag")?;
    Ok(())
  } else {
    unreachable!()
  }
}
