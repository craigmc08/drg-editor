pub mod asset_registry;
pub mod depends;
pub mod exports;
pub mod file_summary;
pub mod imports;
pub mod names;
pub mod preload_dependencies;
pub mod property;
pub mod reference;

pub use asset_registry::*;
pub use depends::*;
pub use exports::*;
pub use file_summary::*;
pub use imports::*;
pub use names::*;
pub use preload_dependencies::*;
pub use property::*;
pub use reference::*;

use crate::reader::*;
use anyhow::*;
use std::io::prelude::Write;
use std::io::Cursor;
use std::path::Path;

pub struct AssetHeader {
  pub summary: FileSummary,
  pub names: Names,
  pub imports: Imports,
  pub exports: Exports,
  pub depends: Depends,
  pub assets: AssetRegistry,
  pub dependencies: PreloadDependencies,
}

pub struct AssetExports {
  pub structs: Vec<Properties>,
}

pub struct Asset {
  pub header: AssetHeader,
  pub exports: AssetExports,
}

impl AssetHeader {
  pub fn read_from(asset_loc: &Path) -> Result<Self> {
    let uasset_fp = asset_loc.with_extension("uasset");

    let uasset = std::fs::read(uasset_fp.clone())
      .with_context(|| format!("Failed to read uasset from {:?}", uasset_fp))?;

    Self::read(uasset)
  }

  pub fn write_out(&self, asset_loc: &Path) -> Result<()> {
    let uasset = self.write()?;

    let uasset_fp = asset_loc.with_extension("uasset");

    std::fs::write(uasset_fp, uasset)?;

    Ok(())
  }

  pub fn read(uasset: Vec<u8>) -> Result<Self> {
    let mut rdr = ByteReader::new(uasset);
    let summary = FileSummary::read(&mut rdr).with_context(|| "Failed to read summary")?;
    let names = Names::read(&mut rdr, &summary).with_context(|| "Failed to read names")?;
    let imports =
      Imports::read(&mut rdr, &summary, &names).with_context(|| "Failed to read imports")?;
    let exports =
      Exports::read(&mut rdr, &summary, &names).with_context(|| "Failed to read exports")?;
    let depends =
      Depends::read(&mut rdr, &summary).with_context(|| "Failed to read dependencies")?;
    let assets =
      AssetRegistry::read(&mut rdr, &summary).with_context(|| "Failed to read asset registry")?;
    let dependencies = PreloadDependencies::read(&mut rdr, &summary, &imports, &exports)
      .with_context(|| "Failed to read preload dependencies")?;
    Ok(Self {
      summary,
      names,
      imports,
      exports,
      depends,
      assets,
      dependencies,
    })
  }

  pub fn write(&self) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(vec![]);
    self
      .summary
      .write(&mut cursor)
      .with_context(|| "Failed to write file summary")?;
    self
      .names
      .write(&mut cursor)
      .with_context(|| "Failed to write names")?;
    self
      .imports
      .write(&mut cursor, &self.names)
      .with_context(|| "Failed to write imports")?;
    self
      .exports
      .write(&mut cursor, &self.names)
      .with_context(|| "Failed to write exports")?;
    self
      .depends
      .write(&mut cursor)
      .with_context(|| "Failed to write dependencies")?;
    self
      .assets
      .write(&mut cursor)
      .with_context(|| "Failed to write asset registry")?;
    self
      .dependencies
      .write(&mut cursor, &self.names, &self.imports, &self.exports)
      .with_context(|| "Failed to write preload dependencies")?;

    Ok(cursor.into_inner())
  }

  /// Recalculates all offsets without using export data
  pub fn recalculate_offsets(&mut self) {
    let names_offset = self.summary.byte_size();
    let import_offset = names_offset + self.names.byte_size();
    let export_offset = import_offset + self.imports.byte_size();
    let deps_offset = export_offset + self.exports.byte_size();
    let assets_offset = deps_offset + self.depends.byte_size();
    let preload_offset = assets_offset + self.assets.byte_size();
    let structs_offset = deps_offset + self.dependencies.byte_size() + 4;
    let total_header_size = structs_offset - 4; // Without the end tag? TODO check this

    let header_size_delta = (structs_offset as i64) - (self.summary.total_header_size as i64);

    self.summary.total_header_size = total_header_size as u32;
    self.summary.name_count = self.names.names.len() as u32;
    self.summary.name_offset = names_offset as u32;
    self.summary.import_count = self.imports.objects.len() as u32;
    self.summary.import_offset = import_offset as u32;
    self.summary.export_count = self.exports.exports.len() as u32;
    self.summary.export_offset = export_offset as u32;
    self.summary.depends_offset = deps_offset as u32;
    self.summary.asset_registry_data_offset = assets_offset as u32;
    self.summary.bulk_data_start_offset =
      (self.summary.bulk_data_start_offset as i64 + header_size_delta) as u32;
    self.summary.preload_dependency_count = self.dependencies.dependencies.len() as u32;
    self.summary.preload_dependency_offset = preload_offset as u32;

    for generation in self.summary.generations.iter_mut() {
      generation.export_count = self.summary.export_count;
      generation.name_count = self.summary.name_count;
    }

    for export in self.exports.exports.iter_mut() {
      // No change to export_file_offset
      // No change to serial size
      export.serial_offset = (export.serial_offset as i64 + header_size_delta) as u32;
    }
  }
}

impl AssetExports {
  pub fn read_from(header: &AssetHeader, asset_loc: &Path) -> Result<Self> {
    let uexp_fp = asset_loc.with_extension("uexp");

    let uexp = std::fs::read(uexp_fp.clone())
      .with_context(|| format!("Failed to read uexp from {:?}", uexp_fp))?;

    Self::read(header, uexp)
  }

  pub fn read(header: &AssetHeader, uexp: Vec<u8>) -> Result<Self> {
    let mut cursor_uexp = ByteReader::new(uexp);
    // Read all export structs
    let mut structs = vec![];
    let ctx = PropertyContext::new(
      &header.summary,
      &header.names,
      &header.imports,
      &header.exports,
      struct_pattern::StructPatterns::get().expect("struct-patterns was not initialized properly"),
    );
    for export in header.exports.exports.iter() {
      let start_pos = cursor_uexp.position();
      let strct = Properties::deserialize(&mut cursor_uexp, export, ctx)
        .with_context(|| format!("Failed to read struct starting at {:#X}", start_pos))?;
      structs.push(strct);
    }
    Ok(AssetExports { structs })
  }

  pub fn write(&self, header: &AssetHeader) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(vec![]);
    let ctx = PropertyContext::new(
      &header.summary,
      &header.names,
      &header.imports,
      &header.exports,
      struct_pattern::StructPatterns::get().expect("struct-patterns was not initialized properly"),
    );
    for (i, strct) in self.structs.iter().enumerate() {
      strct.serialize(&mut cursor, ctx).with_context(|| {
        format!(
          "Failed to write struct {}",
          header.exports.exports[i]
            .object_name
            .to_string(&header.names)
        )
      })?;
    }
    cursor.write_all(&header.summary.tag)?;
    Ok(cursor.into_inner())
  }
}

impl Asset {
  pub fn new(header: AssetHeader, exports: AssetExports) -> Self {
    Self { header, exports }
  }

  pub fn read_from(asset_loc: &Path) -> Result<Self> {
    let uasset_fp = asset_loc.with_extension("uasset");
    let uexp_fp = asset_loc.with_extension("uexp");

    let uasset = std::fs::read(uasset_fp.clone())
      .with_context(|| format!("Failed to read uasset from {:?}", uasset_fp))?;
    let uexp = std::fs::read(uexp_fp.clone())
      .with_context(|| format!("Failed to read uexp from {:?}", uexp_fp))?;

    Self::read(uasset, uexp)
  }

  pub fn write_out(&self, asset_loc: &Path) -> Result<()> {
    let (uasset, uexp) = self.write()?;

    let uasset_fp = asset_loc.with_extension("uasset");
    let uexp_fp = asset_loc.with_extension("uexp");

    std::fs::write(uasset_fp, uasset)?;
    std::fs::write(uexp_fp, uexp)?;

    Ok(())
  }

  pub fn test_rw(asset_loc: &Path) -> Result<()> {
    let uasset_fp = asset_loc.with_extension("uasset");
    let uexp_fp = asset_loc.with_extension("uexp");

    let uasset = std::fs::read(uasset_fp.clone())
      .with_context(|| format!("Failed to read uasset from {:?}", uasset_fp))?;
    let uexp = std::fs::read(uexp_fp.clone())
      .with_context(|| format!("Failed to read uasset from {:?}", uexp_fp))?;

    let asset = Self::read(uasset.clone(), uexp.clone())?;
    let (uasset_out, uexp_out) = asset.write()?;

    if uasset.len() != uasset_out.len() {
      bail!(
        "Different uasset length after writing: {:04X} to {:04X}",
        uasset.len(),
        uasset_out.len()
      )
    }
    if uexp.len() != uexp_out.len() {
      bail!(
        "Different uexo length after writing: {:04X} to {:04X}",
        uexp.len(),
        uexp_out.len()
      )
    }

    for (i, (b1, b2)) in uasset.iter().zip(uasset_out.iter()).enumerate() {
      if b1 != b2 {
        bail!("Different byte in uasset after writing at {:04X}", i);
      }
    }

    for (i, (b1, b2)) in uexp.iter().zip(uexp_out.iter()).enumerate() {
      if b1 != b2 {
        bail!("Different byte in uexp after writing at {:04X}", i);
      }
    }

    Ok(())
  }

  pub fn read(uasset: Vec<u8>, uexp: Vec<u8>) -> Result<Self> {
    let header = AssetHeader::read(uasset)?;
    let exports = AssetExports::read(&header, uexp)?;
    Ok(Self { header, exports })
  }

  pub fn write(&self) -> Result<(Vec<u8>, Vec<u8>)> {
    let uasset = self.header.write()?;
    let uexp = self.exports.write(&self.header)?;
    Ok((uasset, uexp))
  }

  pub fn summary(&self) -> &FileSummary {
    &self.header.summary
  }
  pub fn summary_mut(&mut self) -> &mut FileSummary {
    &mut self.header.summary
  }

  pub fn names(&self) -> &Names {
    &self.header.names
  }
  pub fn names_mut(&mut self) -> &mut Names {
    &mut self.header.names
  }

  pub fn imports(&self) -> &Imports {
    &self.header.imports
  }
  pub fn imports_mut(&mut self) -> &mut Imports {
    &mut self.header.imports
  }

  pub fn exports(&self) -> &Exports {
    &self.header.exports
  }
  pub fn exports_mut(&mut self) -> &mut Exports {
    &mut self.header.exports
  }

  pub fn assets(&self) -> &AssetRegistry {
    &self.header.assets
  }
  pub fn assets_mut(&mut self) -> &mut AssetRegistry {
    &mut self.header.assets
  }

  pub fn deps(&self) -> &PreloadDependencies {
    &self.header.dependencies
  }
  pub fn deps_mut(&mut self) -> &mut PreloadDependencies {
    &mut self.header.dependencies
  }

  pub fn structs(&self) -> &Vec<Properties> {
    &self.exports.structs
  }
  pub fn structs_mut(&mut self) -> &mut Vec<Properties> {
    &mut self.exports.structs
  }

  pub fn recalculate_offsets(&mut self) {
    let structs_size = self
      .structs()
      .iter()
      .map(|ps| ps.byte_size())
      .sum::<usize>();

    self.summary_mut().total_header_size = (self.summary().byte_size()
      + self.names().byte_size()
      + self.imports().byte_size()
      + self.exports().byte_size()
      + self.assets().byte_size()
      + self.deps().byte_size()) as u32;
    self.summary_mut().name_count = self.names().names.len() as u32;
    self.summary_mut().name_offset = self.summary().byte_size() as u32;
    self.summary_mut().export_count = self.exports().exports.len() as u32;
    self.summary_mut().export_offset =
      (self.summary().byte_size() + self.names().byte_size() + self.imports().byte_size()) as u32;
    self.summary_mut().import_count = self.imports().objects.len() as u32;
    self.summary_mut().import_offset =
      (self.summary().byte_size() + self.names().byte_size()) as u32;
    self.summary_mut().depends_offset = (self.summary().byte_size()
      + self.names().byte_size()
      + self.imports().byte_size()
      + self.exports().byte_size()) as u32;
    self.summary_mut().asset_registry_data_offset = (self.summary().byte_size()
      + self.names().byte_size()
      + self.imports().byte_size()
      + self.exports().byte_size()
      + 4) as u32;
    self.summary_mut().bulk_data_start_offset = (self.summary().byte_size()
      + self.names().byte_size()
      + self.imports().byte_size()
      + self.exports().byte_size()
      + self.assets().byte_size()
      + self.deps().byte_size()
      + structs_size) as u32;
    self.summary_mut().preload_dependency_count = self.deps().dependencies.len() as u32;
    self.summary_mut().preload_dependency_offset = (self.summary().byte_size()
      + self.names().byte_size()
      + self.imports().byte_size()
      + self.exports().byte_size()
      + self.assets().byte_size()) as u32;

    // Update all generations counts
    for generation in self.header.summary.generations.iter_mut() {
      generation.export_count = self.header.summary.export_count;
      generation.name_count = self.header.summary.name_count;
    }

    let mut running_size_total = 0;
    for i in 0..(self.summary().export_count as usize) {
      self.exports_mut().exports[i].export_file_offset = running_size_total as u64;
      self.exports_mut().exports[i].serial_size = structs_size as u64;
      self.exports_mut().exports[i].serial_offset =
        running_size_total + self.summary().total_header_size;
      running_size_total += self.exports().exports[i].serial_size as u32;
    }
  }
}
