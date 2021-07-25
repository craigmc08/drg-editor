use crate::asset::*;
use crate::reader::*;
use crate::util::*;
use anyhow::*;
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct Depends {
  pub data: Vec<u8>,
}

impl Depends {
  pub fn read(rdr: &mut ByteReader, summary: &FileSummary) -> Result<Self> {
    if rdr.position() != summary.depends_offset as u64 {
      bail!(
        "Wrong depends starting position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.depends_offset,
        rdr.position(),
      );
    }

    let depends_len = (summary.asset_registry_data_offset - summary.depends_offset) as usize;
    let data = read_bytes(rdr, depends_len)?;
    Ok(Self { data })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    curs.write_all(&self.data[..])?;
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    self.data.len()
  }
}
