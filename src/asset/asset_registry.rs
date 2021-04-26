use crate::asset::*;
use crate::util::*;
use std::io::Cursor;
use std::io::prelude::*;

#[derive(Debug)]
pub struct AssetRegistry {
  // Store bytes of depends and registry sections
  pub depends: Vec<u8>,
  pub registry: Vec<u8>,
}

impl AssetRegistry {
  pub fn read(rdr: &mut Cursor<Vec<u8>>, summary: &FileSummary) -> Result<Self, String> {
    // Asset registry actually contains both Depends and AssetsRegistry
    if rdr.position() != summary.depends_offset.into() {
      return Err(
        format!(
          "Error parsing AssetRegistry: Expected to be at position {:04X}, but I'm at position {:04X}",
          summary.depends_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let depends_len = (summary.asset_registry_data_offset - summary.depends_offset) as usize;
    let depends = read_bytes(rdr, depends_len);
    let assets_len = (summary.preload_dependency_offset - summary.asset_registry_data_offset) as usize;
    let registry = read_bytes(rdr, assets_len);

    return Ok(AssetRegistry { depends, registry });
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>) -> () {
    curs.write(&self.depends[..]).unwrap();
    curs.write(&self.registry[..]).unwrap();
  }

  pub fn byte_size(&self) -> usize {
    8
  }
}