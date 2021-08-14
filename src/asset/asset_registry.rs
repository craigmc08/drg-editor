use crate::asset::*;
use crate::reader::*;
use crate::util::*;
use anyhow::*;
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct AssetRegistry {
  pub data: Vec<u8>,
}

impl AssetRegistry {
  pub fn read(rdr: &mut ByteReader, summary: &FileSummary) -> Result<Self> {
    // Asset registry actually contains both Depends and AssetsRegistry
    if rdr.position() != summary.asset_registry_data_offset as u64 {
      bail!(
        "Wrong asset registry starting position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.depends_offset,
        rdr.position(),
      );
    }

    let assets_len =
      (summary.preload_dependency_offset - summary.asset_registry_data_offset) as usize;
    let data = read_bytes(rdr, assets_len)?;
    Ok(AssetRegistry { data })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    curs.write_all(&self.data[..])?;
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    self.data.len()
  }
}
