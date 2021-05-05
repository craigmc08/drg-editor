use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::meta::*;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader;
use crate::reader::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;

pub const LOADER_ARRAY: PropertyLoader = loader!(
  [PropType::ArrayProperty],
  deserialize_array,
  deserialize_array_tag,
  serialize_array,
  serialize_array_tag,
  value_size_array,
  |_| 8,
);

fn deserialize_array_tag(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Tag> {
  let inner_type = PropType::deserialize(rdr, ctx)?;
  Ok(Tag::Array { inner_type })
}

/// # Panics
/// If `tag` is not Array variant.
fn serialize_array_tag(tag: &Tag, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
  if let Tag::Array { inner_type } = tag {
    inner_type.serialize(curs, ctx)?;
    Ok(())
  } else {
    unreachable!()
  }
}

/// # Panics
/// If `tag` is not Array variant.
fn deserialize_array(
  rdr: &mut ByteReader,
  tag: &Tag,
  max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  if let Tag::Array { inner_type } = tag {
    let count = rdr.read_u32::<LittleEndian>()?;
    let loader = Property::get_loader_for(*inner_type).with_context(|| "Array.inner_type")?;

    let (inner_meta, inner_tag, inner_max_size) = if loader.simple {
      (None, Tag::Simple(*inner_type), max_size)
    } else {
      // TODO: might want to .ok the meta deserialization.
      // not sure if this will cause problems how it is.
      let meta = Meta::deserialize(rdr, ctx)?;
      let inner_tag = (loader.deserialize_tag)(rdr, ctx)?;
      rdr.read_exact(&mut [0])?;
      let inner_max_size = meta.as_ref().map(|m| m.size).unwrap_or(0); // TODO this unwrap shouldn't fail?
      (meta, inner_tag, inner_max_size)
    };

    let mut values = vec![];
    for i in 0..count {
      let value = (loader.deserialize_value)(rdr, &inner_tag, inner_max_size, ctx)
        .with_context(|| format!("Array[{}]", i))?;
      values.push(value);
    }

    Ok(Value::Array {
      inner_meta,
      inner_tag,
      values,
    })
  } else {
    unreachable!()
  }
}

/// # Panics
/// If `val` is not Array variant or `tag` is not Array variant.
fn serialize_array(
  val: &Value,
  tag: &Tag,
  curs: &mut Cursor<Vec<u8>>,
  ctx: PropertyContext,
) -> Result<()> {
  if let (
    Value::Array {
      inner_meta,
      inner_tag,
      values,
    },
    Tag::Array { inner_type },
  ) = (val, tag)
  {
    let loader = Property::get_loader_for(*inner_type).with_context(|| "Array.inner_type")?;
    if let Some(meta) = inner_meta {
      meta.serialize(curs, ctx)?;
    }
    (loader.serialize_tag)(inner_tag, curs, ctx)?;
    let len = values.len();
    curs.write_u32::<LittleEndian>(len as u32)?;
    for (i, value) in values.iter().enumerate() {
      (loader.serialize_value)(value, inner_tag, curs, ctx)
        .with_context(|| format!("Array[{}]", i))?;
    }
    Ok(())
  } else {
    unreachable!()
  }
}

fn value_size_array(value: &Value, tag: &Tag) -> usize {
  if let (
    Value::Array {
      inner_meta,
      inner_tag,
      values,
    },
    Tag::Array { inner_type },
  ) = (value, tag)
  {
    let loader = Property::get_loader_for(*inner_type).expect("Unreachable");
    let meta_size = inner_meta.as_ref().map(Meta::byte_size).unwrap_or(0);
    let tag_size = (loader.tag_size)(&inner_tag);
    let values_size = values
      .iter()
      .map(|v| (loader.value_size)(v, &inner_tag))
      .sum::<usize>();

    // Meta + tag + length + values
    meta_size + tag_size + 4 + values_size
  } else {
    unreachable!()
  }
}
