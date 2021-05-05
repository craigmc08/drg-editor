use crate::asset::property::context::*;
use crate::asset::property::loaders::PropertyLoader;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::loader_simple;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub const LOADER_INT: PropertyLoader = loader_simple!(
  PropType::IntProperty,
  deserialize_int,
  serialize_int,
  |_, _| 4,
);
pub const LOADER_FLOAT: PropertyLoader = loader_simple!(
  PropType::FloatProperty,
  deserialize_float,
  serialize_float,
  |_, _| 4,
);
pub const LOADER_OBJECT: PropertyLoader = loader_simple!(
  PropType::ObjectProperty,
  deserialize_object,
  serialize_object,
  |_, _| 4,
);
pub const LOADER_SOFTOBJECT: PropertyLoader = loader_simple!(
  PropType::SoftObjectProperty,
  deserialize_softobject,
  serialize_softobject,
  |_, _| 12,
);
pub const LOADER_NAME: PropertyLoader = loader_simple!(
  PropType::NameProperty,
  deserialize_name,
  serialize_name,
  |_, _| 8,
);

// TODO: figure out if boiler plate on the numeric types can be reduced further?

fn deserialize_int(rdr: Curs, _tag: &Tag, _max_size: u64, _ctx: PropertyContext) -> Result<Value> {
  Ok(Value::Int(rdr.read_i32::<LittleEndian>()?))
}
/// # Panics
/// If `val` is not Int variant.
fn serialize_int(val: &Value, _tag: &Tag, curs: Curs, _ctx: PropertyContext) -> Result<()> {
  if let Value::Int(val) = val {
    curs.write_i32::<LittleEndian>(*val)?;
    Ok(())
  } else {
    unreachable!()
  }
}

fn deserialize_float(
  rdr: Curs,
  _tag: &Tag,
  _max_size: u64,
  _ctx: PropertyContext,
) -> Result<Value> {
  Ok(Value::Float(rdr.read_f32::<LittleEndian>()?))
}
/// # Panics
/// If `val` is not Float variant.
fn serialize_float(val: &Value, _tag: &Tag, curs: Curs, _ctx: PropertyContext) -> Result<()> {
  if let Value::Float(val) = val {
    curs.write_f32::<LittleEndian>(*val)?;
    Ok(())
  } else {
    unreachable!()
  }
}

fn deserialize_object(
  rdr: Curs,
  _tag: &Tag,
  _max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  Ok(Value::Object(Dependency::read(
    rdr,
    ctx.imports,
    ctx.exports,
  )?))
}
/// # Panics
/// If `val` is not Object variant.
fn serialize_object(val: &Value, _tag: &Tag, curs: Curs, ctx: PropertyContext) -> Result<()> {
  if let Value::Object(dep) = val {
    dep.write(curs, ctx.imports, ctx.exports)?;
    Ok(())
  } else {
    unreachable!()
  }
}

fn deserialize_softobject(
  rdr: Curs,
  _tag: &Tag,
  _max_size: u64,
  ctx: PropertyContext,
) -> Result<Value> {
  Ok(Value::SoftObject {
    object_name: NameVariant::read(rdr, ctx.names).with_context(|| "SoftObject.object_name")?,
    unk1: rdr.read_u32::<LittleEndian>()?,
  })
}
/// # Panics
/// If `val` is not SoftObject variant
fn serialize_softobject(val: &Value, _tag: &Tag, curs: Curs, ctx: PropertyContext) -> Result<()> {
  if let Value::SoftObject { object_name, unk1 } = val {
    object_name
      .write(curs, ctx.names)
      .with_context(|| "SoftObject.object_name")?;
    curs.write_u32::<LittleEndian>(*unk1)?;
    Ok(())
  } else {
    unreachable!()
  }
}

fn deserialize_name(rdr: Curs, _: &Tag, _: u64, ctx: PropertyContext) -> Result<Value> {
  Ok(Value::Name(
    NameVariant::read(rdr, ctx.names).with_context(|| "Name")?,
  ))
}
/// # Panics
/// If val is not Name variant
fn serialize_name(val: &Value, _: &Tag, curs: Curs, ctx: PropertyContext) -> Result<()> {
  if let Value::Name(val) = val {
    val.write(curs, ctx.names).with_context(|| "Names")?;
    Ok(())
  } else {
    unreachable!()
  }
}