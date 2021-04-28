use crate::asset::*;
use crate::util::*;
use anyhow::*;
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct AssetRegistry {
  // Store bytes of depends and registry sections
  pub depends: Vec<u8>,
  pub registry: Vec<u8>,
}

impl AssetRegistry {
  pub fn read(rdr: &mut Cursor<Vec<u8>>, summary: &FileSummary) -> Result<Self> {
    // Asset registry actually contains both Depends and AssetsRegistry
    if rdr.position() != summary.depends_offset.into() {
      bail!(
        "Wrong asset registry starting position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.depends_offset,
        rdr.position(),
      );
    }

    let depends_len = (summary.asset_registry_data_offset - summary.depends_offset) as usize;
    let depends = read_bytes(rdr, depends_len)?;
    let assets_len =
      (summary.preload_dependency_offset - summary.asset_registry_data_offset) as usize;
    let registry = read_bytes(rdr, assets_len)?;

    return Ok(AssetRegistry { depends, registry });
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    curs.write(&self.depends[..])?;
    curs.write(&self.registry[..])?;
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    8
  }
}
