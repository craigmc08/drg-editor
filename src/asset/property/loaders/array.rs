use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::meta::*;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader;
use crate::reader::*;
use crate::util::*;
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

/// Current version does not attempt to parse non-simple inner values, e.g.
/// Struct. In this case, it will return a RawData value.
///
/// # Panics
/// If `tag` is not Array variant.
fn deserialize_array(
  rdr: &mut ByteReader,
  tag: &Tag,
  max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  if let Tag::Array { inner_type } = tag {
    let loader = Property::get_loader_for(*inner_type).with_context(|| "Array.inner_type")?;

    if loader.simple {
      let count = rdr.read_u32::<LittleEndian>()?;
      let inner_tag = Tag::Simple(*inner_type);
      let mut values = vec![];
      for i in 0..count {
        let value = (loader.deserialize_value)(rdr, &inner_tag, max_size, ctx)
          .with_context(|| format!("Array[{}]", i))?;
        values.push(value);
      }
      Ok(Value::Array { values })
    } else {
      let data: Vec<u8> = read_bytes(rdr, max_size as usize)?;
      Ok(Value::RawData { data })
    }
  } else {
    unreachable!()
  }
}

/// # Panics
/// If `val` is not Array or RawData variant or `tag` is not Array variant.
fn serialize_array(
  val: &Value,
  tag: &Tag,
  curs: &mut Cursor<Vec<u8>>,
  ctx: PropertyContext,
) -> Result<()> {
  match (val, tag) {
    (Value::Array { values }, Tag::Array { inner_type }) => {
      let inner_tag = Tag::Simple(*inner_type);
      let loader = Property::get_loader_for(*inner_type).with_context(|| "Array.inner_type")?;
      let len = values.len();
      curs.write_u32::<LittleEndian>(len as u32)?;
      for (i, value) in values.iter().enumerate() {
        loader
          .serialize_value(curs, value, &inner_tag, ctx)
          .with_context(|| format!("Array[{}]", i))?;
      }
      Ok(())
    }
    (Value::RawData { data }, _) => {
      curs.write(data)?;
      Ok(())
    }
    _ => {
      unreachable!()
    }
  }
}

/// # Panics
/// Panics if value is not Array or RawData or Tag is not Array
fn value_size_array(value: &Value, tag: &Tag) -> usize {
  match (value, tag) {
    (Value::Array { values }, Tag::Array { inner_type }) => {
      let inner_tag = Tag::Simple(*inner_type);
      let loader = Property::get_loader_for(*inner_type).expect("Unreachable");
      let values_size = values
        .iter()
        .map(|v| (loader.value_size)(v, &inner_tag))
        .sum::<usize>();
      values_size
    }
    (Value::RawData { data }, _) => data.len(),
    _ => {
      unreachable!()
    }
  }
}
