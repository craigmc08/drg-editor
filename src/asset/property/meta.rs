use crate::asset::property::context::*;
use crate::asset::property::prop_type::*;
use crate::asset::*;
use crate::reader::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone)]
pub struct Meta {
  pub name: NameVariant,
  pub typ: PropType,
  pub size: u64,
}

impl Meta {
  pub fn new(name: impl Into<NameVariant>, typ: PropType, size: u64) -> Self {
    Self {
      name: name.into(),
      typ,
      size,
    }
  }

  pub fn deserialize(rdr: &mut ByteReader, ctx: PropertyContext) -> Result<Option<Self>> {
    let name = NameVariant::read(rdr, ctx.names)?;
    if name.to_string() == "None" {
      return Ok(None);
    }

    let typ = PropType::deserialize(rdr, ctx)?;
    let size = rdr.read_u64::<LittleEndian>()?;
    Ok(Some(Self { name, typ, size }))
  }

  pub fn serialize(&self, curs: &mut Cursor<Vec<u8>>, ctx: PropertyContext) -> Result<()> {
    self.name.write(curs, ctx.names)?;
    self.typ.serialize(curs, ctx)?;
    curs.write_u64::<LittleEndian>(self.size)?;
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    24
  }

  pub fn maybe_byte_size(meta: &Option<Self>) -> usize {
    meta.as_ref().map(Self::byte_size).unwrap_or(8)
  }
}
