pub mod asset_registry;
pub mod export_map;
pub mod file_summary;
pub mod name_map;
pub mod object_imports;
pub mod preload_dependencies;
pub mod property;

pub use asset_registry::*;
pub use export_map::*;
pub use file_summary::*;
pub use name_map::*;
pub use object_imports::*;
pub use preload_dependencies::*;
pub use property::*;

use crate::reader::*;
use anyhow::*;
use std::io::prelude::Write;
use std::io::Cursor;
use std::path::Path;

pub struct Asset {
  pub summary: FileSummary,
  pub names: NameMap,
  pub imports: ObjectImports,
  pub exports: ObjectExports,
  pub assets: AssetRegistry,
  pub dependencies: PreloadDependencies,
  pub structs: Vec<Properties>,
}

impl Asset {
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

  pub fn read(uasset: Vec<u8>, uexp: Vec<u8>) -> Result<Self> {
    let mut cursor_uasset = ByteReader::new(uasset);
    let summary =
      FileSummary::read(&mut cursor_uasset).with_context(|| format!("Failed to read summary"))?;
    let names = NameMap::read(&mut cursor_uasset, &summary)
      .with_context(|| format!("Failed to read names"))?;
    let imports = ObjectImports::read(&mut cursor_uasset, &summary, &names)
      .with_context(|| format!("Failed to read imports"))?;
    let exports = ObjectExports::read(&mut cursor_uasset, &summary, &names, &imports)
      .with_context(|| format!("Failed to read exports"))?;
    let assets = AssetRegistry::read(&mut cursor_uasset, &summary)
      .with_context(|| format!("Failed to read dependencies or asset registry"))?;
    let dependencies = PreloadDependencies::read(&mut cursor_uasset, &summary, &imports, &exports)
      .with_context(|| format!("Failed to read preload dependencies"))?;

    let mut cursor_uexp = ByteReader::new(uexp);
    // Read all export structs
    let mut structs = vec![];
    let ctx = PropertyContext::new(&summary, &names, &imports, &exports);
    for export in exports.exports.iter() {
      let start_pos = cursor_uexp.position();
      let strct = Properties::deserialize(&mut cursor_uexp, export, ctx)
        .with_context(|| format!("Failed to read struct starting at {:#X}", start_pos))?;
      structs.push(strct);
    }

    Ok(Asset {
      summary,
      names,
      imports,
      exports,
      assets,
      dependencies,
      structs,
    })
  }

  pub fn write(&self) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut cursor_uasset = Cursor::new(vec![]);
    self
      .summary
      .write(&mut cursor_uasset)
      .with_context(|| "Failed to write file summary")?;
    self
      .names
      .write(&mut cursor_uasset)
      .with_context(|| "Failed to write names")?;
    self
      .imports
      .write(&mut cursor_uasset, &self.names)
      .with_context(|| "Failed to write imports")?;
    self
      .exports
      .write(&mut cursor_uasset, &self.names, &self.imports)
      .with_context(|| "Failed to write exports")?;
    self
      .assets
      .write(&mut cursor_uasset)
      .with_context(|| "Failed to write dependencies or asset registry")?;
    self
      .dependencies
      .write(&mut cursor_uasset, &self.imports, &self.exports)
      .with_context(|| "Failed to write preload dependencies")?;

    let mut cursor_uexp = Cursor::new(vec![]);
    let ctx = PropertyContext::new(&self.summary, &self.names, &self.imports, &self.exports);
    for (i, strct) in self.structs.iter().enumerate() {
      strct.serialize(&mut cursor_uexp, ctx).with_context(|| {
        format!(
          "Failed to write struct {}",
          self.exports.exports[i].object_name
        )
      })?;
    }
    cursor_uexp.write(&self.summary.tag)?;

    Ok((
      cursor_uasset.get_ref().clone(),
      cursor_uexp.get_ref().clone(),
    ))
  }

  pub fn recalculate_offsets(&mut self) -> () {
    let structs_size = self.structs.iter().map(|ps| ps.byte_size()).sum::<usize>();

    self.summary.total_header_size = (self.summary.byte_size()
      + self.names.byte_size()
      + self.imports.byte_size()
      + self.exports.byte_size()
      + self.assets.byte_size()
      + self.dependencies.byte_size()) as u32;
    self.summary.name_count = self.names.names.len() as u32;
    self.summary.name_offset = self.summary.byte_size() as u32;
    self.summary.export_count = self.exports.exports.len() as u32;
    self.summary.export_offset =
      (self.summary.byte_size() + self.names.byte_size() + self.imports.byte_size()) as u32;
    self.summary.import_count = self.imports.objects.len() as u32;
    self.summary.import_offset = (self.summary.byte_size() + self.names.byte_size()) as u32;
    self.summary.depends_offset = (self.summary.byte_size()
      + self.names.byte_size()
      + self.imports.byte_size()
      + self.exports.byte_size()) as u32;
    self.summary.asset_registry_data_offset = (self.summary.byte_size()
      + self.names.byte_size()
      + self.imports.byte_size()
      + self.exports.byte_size()
      + 4) as u32;
    self.summary.bulk_data_start_offset = (self.summary.byte_size()
      + self.names.byte_size()
      + self.imports.byte_size()
      + self.exports.byte_size()
      + self.assets.byte_size()
      + self.dependencies.byte_size()
      + structs_size) as u32;
    self.summary.preload_dependency_count = self.dependencies.dependencies.len() as u32;
    self.summary.preload_dependency_offset = (self.summary.byte_size()
      + self.names.byte_size()
      + self.imports.byte_size()
      + self.exports.byte_size()
      + self.assets.byte_size()) as u32;

    // Update all generations counts
    for generation in self.summary.generations.iter_mut() {
      generation.export_count = self.summary.export_count;
      generation.name_count = self.summary.name_count;
    }

    let mut running_size_total = 0;
    for i in 0..(self.summary.export_count as usize) {
      self.exports.exports[i].export_file_offset = running_size_total as u64;
      self.exports.exports[i].serial_size = structs_size as u64;
      self.exports.exports[i].serial_offset = running_size_total + self.summary.total_header_size;
      running_size_total += self.exports.exports[i].serial_size as u32;
    }
  }
}
