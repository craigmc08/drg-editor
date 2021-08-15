use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader;
use crate::reader::*;
use crate::util::*;
use std::io::prelude::*;

pub const LOADER_STRUCT: PropertyLoader = loader!(
  [PropType::StructProperty],
  deserialize_struct,
  deserialize_struct_tag,
  serialize_struct,
  serialize_struct_tag,
  value_size_struct,
  |_| 24,
);

fn deserialize_struct_tag(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Tag> {
  let type_name = NameVariant::read(rdr, ctx.names).with_context(|| "Struct.type_name")?;
  let guid = read_bytes(rdr, 16)?;
  Ok(Tag::Struct { type_name, guid })
}

/// # Panics
/// If `tag` is not Struct variant.
fn serialize_struct_tag(tag: &Tag, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
  if let Tag::Struct { type_name, guid } = tag {
    type_name
      .write(curs, ctx.names)
      .with_context(|| "Struct.type_name")?;
    curs.write_all(guid)?;
    Ok(())
  } else {
    unreachable!()
  }
}

/// # Panics
/// If `tag` is not Struct variant
fn deserialize_struct(
  rdr: &mut ByteReader,
  tag: &Tag,
  _max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  match tag {
    Tag::Struct { type_name, .. } => {
      let struct_type = type_name.to_string(ctx.names);
      let value = ctx.patterns.deserialize(rdr, &struct_type, ctx)?;
      Ok(Value::Struct { value })
    }
    _ => unreachable!(),
  }
}

/// # Panics
/// Panics if `val` and `tag` are not Struct variant.s
fn serialize_struct(
  val: &Value,
  _tag: &Tag,
  curs: &mut Cursor<Vec<u8>>,
  ctx: PropertyContext,
) -> Result<()> {
  match val {
    Value::Struct { value } => {
      value.serialize(curs, ctx)?;
      Ok(())
    }
    _ => unreachable!(),
  }
}

/// # Panics
/// If `value` is not Struct variant.
fn value_size_struct(value: &Value, _tag: &Tag) -> usize {
  match value {
    Value::Struct { value } => value.byte_size(),
    _ => unreachable!(),
  }
}
