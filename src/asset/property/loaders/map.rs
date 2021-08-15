use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader;
use crate::reader::*;
use crate::util::*;
use std::io::prelude::*;

pub const LOADER_MAP: PropertyLoader = loader!(
  [PropType::MapProperty],
  deserialize_map,
  deserialize_map_tag,
  serialize_map,
  serialize_map_tag,
  value_size_map,
  |_| 24,
);

fn deserialize_map_tag(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Tag> {
  let key_type = PropType::deserialize(rdr, ctx).with_context(|| "map.key_type")?;
  let value_type = PropType::deserialize(rdr, ctx).with_context(|| "map.value_type")?;
  Ok(Tag::Map {
    key_type,
    value_type,
  })
}

/// # Panics
/// If `tag` is not Map variant.
fn serialize_map_tag(tag: &Tag, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
  if let Tag::Map {
    key_type,
    value_type,
  } = tag
  {
    key_type
      .serialize(curs, ctx)
      .with_context(|| "map.key_type")?;
    value_type
      .serialize(curs, ctx)
      .with_context(|| "map.value_type")?;
    Ok(())
  } else {
    unreachable!()
  }
}

/// # Panics
/// If `tag` is not Map variant
fn deserialize_map(
  rdr: &mut ByteReader,
  tag: &Tag,
  max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  match tag {
    Tag::Map {
      key_type,
      value_type,
    } => {
      let key_loader = Property::get_loader_for(*key_type).with_context(|| "Map.key_type")?;
      let value_loader = Property::get_loader_for(*value_type).with_context(|| "Map.value_type")?;

      if key_loader.simple && value_loader.simple {
        let key_tag = Tag::Simple(*key_type);
        let value_tag = Tag::Simple(*value_type);

        // TODO: what to do with this?
        let num_keys_to_remove = read_u32(rdr).with_context(|| "Map.num_keys_to_remove")?;

        let num_entries = read_u32(rdr).with_context(|| "Map.num_entries")?;
        let mut entries = vec![];
        for i in 0..num_entries {
          let key = key_loader
            .deserialize_value(rdr, &key_tag, max_size, ctx)
            .with_context(|| format!("Map.key[{}]", i))?;
          let value = value_loader
            .deserialize_value(rdr, &value_tag, max_size, ctx)
            .with_context(|| format!("Map.value[{}]", i))?;
          entries.push((key, value));
        }
        Ok(Value::Map {
          num_keys_to_remove,
          entries,
        })
      } else {
        let data: Vec<u8> =
          read_bytes(rdr, max_size as usize).with_context(|| "Map of complex data")?;
        Ok(Value::RawData { data })
      }
    }
    _ => unreachable!(),
  }
}

/// # Panics
/// Panics if `val` and `tag` are not Map or RawData variants
fn serialize_map(
  val: &Value,
  tag: &Tag,
  curs: &mut Cursor<Vec<u8>>,
  ctx: PropertyContext,
) -> Result<()> {
  match (val, tag) {
    (
      Value::Map {
        num_keys_to_remove,
        entries,
      },
      Tag::Map {
        key_type,
        value_type,
      },
    ) => {
      let key_loader = Property::get_loader_for(*key_type).with_context(|| "Map.key_type")?;
      let value_loader = Property::get_loader_for(*value_type).with_context(|| "Map.value_type")?;

      if !key_loader.simple || !value_loader.simple {
        // Enforced by deserialize_map
        unreachable!();
      }
      let key_tag = Tag::Simple(*key_type);
      let value_tag = Tag::Simple(*value_type);

      write_u32(curs, *num_keys_to_remove)?;
      write_u32(curs, entries.len() as u32)?;
      for (i, (key, value)) in entries.iter().enumerate() {
        key_loader
          .serialize_value(curs, key, &key_tag, ctx)
          .with_context(|| format!("Map.key[{}]", i))?;
        value_loader
          .serialize_value(curs, value, &value_tag, ctx)
          .with_context(|| format!("Map.value[{}]", i))?;
      }
      Ok(())
    }
    (Value::RawData { data }, _) => {
      curs
        .write_all(data)
        .with_context(|| "Map of complex data")?;
      Ok(())
    }
    _ => unreachable!(),
  }
}

/// # Panics
/// If `value` is not map variant.
fn value_size_map(value: &Value, tag: &Tag) -> usize {
  match (value, tag) {
    (
      Value::Map { entries, .. },
      Tag::Map {
        key_type,
        value_type,
      },
    ) => {
      let key_loader = Property::get_loader_for(*key_type)
        .with_context(|| "Map.key_type")
        .expect("unreachable");
      let value_loader = Property::get_loader_for(*value_type)
        .with_context(|| "Map.value_type")
        .expect("unreachable");

      if !key_loader.simple || !value_loader.simple {
        // Enforced by deserialize_map
        unreachable!();
      }

      let key_tag = Tag::Simple(*key_type);
      let value_tag = Tag::Simple(*value_type);

      let entries_size: usize = entries
        .iter()
        .map(|(k, v)| key_loader.value_size(k, &key_tag) + value_loader.value_size(v, &value_tag))
        .sum();

      4 + 4 + entries_size // num_keys_to_remove + num_entries + entries
    }
    (Value::RawData { data }, _) => data.len(),
    _ => unreachable!(),
  }
}
