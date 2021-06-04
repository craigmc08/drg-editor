mod array;
mod other;
mod simple;
mod strct;

use array::*;
use other::*;
use simple::*;
use strct::*;

use crate::asset::property::context::*;
use crate::asset::property::prop_type::*;
use crate::asset::property::{Tag, Value};
use crate::reader::*;
use anyhow::*;
use std::io::Cursor;

pub const LOADERS: &[PropertyLoader] = &[
  LOADER_INT,
  LOADER_FLOAT,
  LOADER_OBJECT,
  LOADER_SOFTOBJECT,
  LOADER_NAME,
  LOADER_TEXT,
  LOADER_BOOL,
  LOADER_ENUM,
  LOADER_ARRAY,
  LOADER_STRUCT,
];

type TagDeserializer = dyn Fn(&mut ByteReader, PropertyContext) -> Result<Tag>;
type ValueDeserializer = dyn Fn(&mut ByteReader, &Tag, u64, PropertyContext) -> Result<Value>;
type TagSerializer = dyn Fn(&Tag, &mut Cursor<Vec<u8>>, PropertyContext) -> Result<()>;
type ValueSerializer = dyn Fn(&Value, &Tag, &mut Cursor<Vec<u8>>, PropertyContext) -> Result<()>;
type ValueSizer = dyn Fn(&Value, &Tag) -> usize;
type TagSizer = dyn Fn(&Tag) -> usize;

pub struct PropertyLoader<'a> {
  /// If true, uses Tag::Simple
  pub simple: bool,
  pub for_types: &'a [PropType],
  deserialize_value: &'a ValueDeserializer,
  deserialize_tag: &'a TagDeserializer,
  serialize_value: &'a ValueSerializer,
  serialize_tag: &'a TagSerializer,
  value_size: &'a ValueSizer,
  tag_size: &'a TagSizer,
}

#[macro_export]
macro_rules! loader {
  ( $typs:expr , $dv:expr , $dt:expr, $sv:expr, $st:expr, $vs:expr, $ts:expr $(,)? ) => {
    PropertyLoader {
      simple: false,
      for_types: &$typs,
      deserialize_value: &$dv,
      deserialize_tag: &$dt,
      serialize_value: &$sv,
      serialize_tag: &$st,
      value_size: &$vs,
      tag_size: &$ts,
    }
  };
}

#[macro_export]
macro_rules! loader_simple {
  ( $typ:expr , $dv:expr , $sv:expr , $vs:expr $(,)? ) => {
    PropertyLoader {
      simple: true,
      for_types: &[$typ],
      deserialize_value: &$dv,
      serialize_value: &$sv,
      value_size: &$vs,
      deserialize_tag: &|_, _| Ok(Tag::Simple($typ)),
      serialize_tag: &|_, _, _| Ok(()),
      tag_size: &|_| 0,
    }
  };
}

impl<'a> PropertyLoader<'a> {
  pub fn is_for_type(&self, typ: PropType) -> bool {
    self.for_types.iter().any(|t| *t == typ)
  }

  pub fn deserialize_value(
    &self,
    rdr: &mut ByteReader,
    tag: &Tag,
    max_size: u64,
    ctx: PropertyContext,
  ) -> Result<Value> {
    (self.deserialize_value)(rdr, tag, max_size, ctx)
  }

  pub fn deserialize_tag(&self, rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Tag> {
    (self.deserialize_tag)(rdr, ctx)
  }

  pub fn serialize_value(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    value: &Value,
    tag: &Tag,
    ctx: PropertyContext,
  ) -> Result<()> {
    (self.serialize_value)(value, tag, curs, ctx)
  }

  pub fn serialize_tag(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    tag: &Tag,
    ctx: PropertyContext,
  ) -> Result<()> {
    (self.serialize_tag)(tag, curs, ctx)
  }

  pub fn value_size(&self, value: &Value, tag: &Tag) -> usize {
    (self.value_size)(value, tag)
  }

  pub fn tag_size(&self, tag: &Tag) -> usize {
    (self.tag_size)(tag)
  }
}
