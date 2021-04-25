use crate::file_summary::*;
use crate::util::*;
use std::io::Cursor;
use std::io::BufRead;

#[derive(Debug)]
pub struct AssetRegistry {
  size: u32, // seems to always be 0?
}

impl AssetRegistry {
  pub fn read(rdr: &mut Cursor<Vec<u8>>, summary: &FileSummary) -> Result<Self, String> {
    // Lazily didn't make Dependencies class, so consume it here. This would break
    // if there are any dependencies
    rdr.consume(4);
    if rdr.position() != summary.asset_registry_data_offset.into() {
      return Err(
        format!(
          "Error parsing AssetRegistry: Expected to be at position {}, but I'm at position {}",
          summary.asset_registry_data_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let size = read_u32(rdr);
    return Ok(AssetRegistry { size });
  }

  pub fn byte_size(&self) -> usize {
    8
  }
}
