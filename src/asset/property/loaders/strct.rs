use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader;
use crate::reader::*;
use crate::util::*;
use std::io::prelude::*;
use std::io::{Seek, SeekFrom};

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
    curs.write(guid)?;
    Ok(())
  } else {
    unreachable!()
  }
}

/// # Panics
/// If `tag` is not Struct variant
fn deserialize_struct(
  rdr: &mut ByteReader,
  _tag: &Tag,
  max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  let start_pos = rdr.position();
  let end_pos = rdr.position() + max_size;

  let mut read_props = || -> Result<Vec<Property>> {
    let mut properties = vec![];
    let mut i = 0;
    'structloop: while rdr.position() < end_pos {
      let start_pos = rdr.position();
      let property = Property::deserialize(rdr, ctx)
        .with_context(|| format!("Struct[{}] at {:#X}", i, start_pos))?;
      i += 1;
      if let Some(property) = property {
        properties.push(property);
      } else {
        break 'structloop;
      }
    }
    Ok(properties)
  };

  if let Ok(properties) = read_props() {
    Ok(Value::Struct { properties })
  } else {
    rdr.seek(SeekFrom::Start(start_pos))?;
    let mut none: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let none_name: NameVariant = "None".into();
    none_name
      .write(&mut none, ctx.names)
      .with_context(|| "Expected None in names")?;
    let none = &none.into_inner()[..];
    // Read raw struct if failed to read properties
    let mut data = vec![];
    'dataloop: while rdr.position() < end_pos {
      if next_matches(rdr, none) {
        let read: Vec<u8> = read_bytes(rdr, none.len())?;
        data.extend(read);
        break 'dataloop;
      } else {
        let mut read: [u8; 1] = [0];
        rdr.read_exact(&mut read)?;
        data.extend(read.iter());
      }
    }
    Ok(Value::RawData { data })
  }
}

/// # Panics
/// Panics if `val` is not Struct or RawData variant.
fn serialize_struct(
  val: &Value,
  _tag: &Tag,
  curs: &mut Cursor<Vec<u8>>,
  ctx: PropertyContext,
) -> Result<()> {
  match val {
    Value::Struct { properties } => {
      for (i, property) in properties.iter().enumerate() {
        property
          .serialize(curs, ctx)
          .with_context(|| format!("Struct[{}] or Struct['{}']", i, property.meta.name))?;
      }
      // Write None property
      let none: NameVariant = "None".into();
      none
        .write(curs, ctx.names)
        .with_context(|| "Expected None in names")?;
      Ok(())
    }
    Value::RawData { data } => {
      curs.write(data)?;
      Ok(())
    }
    _ => unreachable!(),
  }
}

/// # Panics
/// If `value` is not Struct or RawData variant.
fn value_size_struct(value: &Value, _tag: &Tag) -> usize {
  match value {
    Value::Struct { properties } => {
      let props_size = properties
        .iter()
        .map(|prop| prop.byte_size())
        .sum::<usize>();
      // Include 8 byte name attribute
      props_size + 8
    }
    Value::RawData { data } => data.len(),
    _ => unreachable!(),
  }
}
