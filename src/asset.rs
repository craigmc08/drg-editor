pub mod file_summary;
pub mod name_map;
pub mod object_imports;
pub mod export_map;
pub mod asset_registry;
pub mod preload_dependencies;
pub mod property;

pub use file_summary::*;
pub use name_map::*;
pub use object_imports::*;
pub use export_map::*;
pub use asset_registry::*;
pub use preload_dependencies::*;
pub use property::*;

use std::io::Cursor;

pub struct Asset {
  pub summary: FileSummary,
  pub names: NameMap,
  pub imports: ObjectImports,
  pub exports: ObjectExports,
  pub assets: AssetRegistry,
  pub dependencies: PreloadDependencies,
  pub properties: Vec<Property>,
}

impl Asset {
  pub fn read(uasset: Vec<u8>, uexp: Vec<u8>) -> Result<Self, String> {
    let mut cursor_uasset = Cursor::new(uasset);
    let summary = FileSummary::read(&mut cursor_uasset);
    let names = NameMap::read(&mut cursor_uasset, &summary)?;
    let imports = ObjectImports::read(&mut cursor_uasset, &summary, &names)?;
    let exports = ObjectExports::read(&mut cursor_uasset, &summary, &names, &imports)?;
    let assets = AssetRegistry::read(&mut cursor_uasset, &summary)?;
    let dependencies = PreloadDependencies::read(&mut cursor_uasset, &summary, &imports)?;

    let mut cursor_uexp = Cursor::new(uexp);
    let properties = Property::read_uexp(&mut cursor_uexp, &names, &imports)?;

    return Ok(Asset {
      summary,
      names,
      imports,
      exports,
      assets,
      dependencies,
      properties
    })
  }

  pub fn write(&self) -> (Vec<u8>, Vec<u8>) {
    let mut cursor_uasset = Cursor::new(vec!());
    self.summary.write(&mut cursor_uasset);
    self.names.write(&mut cursor_uasset);
    self.imports.write(&mut cursor_uasset, &self.names);
    self.exports.write(&mut cursor_uasset, &self.names, &self.imports);
    self.assets.write(&mut cursor_uasset);
    self.dependencies.write(&mut cursor_uasset, &self.imports);

    let mut cursor_uexp = Cursor::new(vec!());
    Property::write_uexp(&self.properties, &mut cursor_uexp, &self.summary, &self.names, &self.imports);

    return (cursor_uasset.get_ref().clone(), cursor_uexp.get_ref().clone())
  }

  pub fn recalculate_offsets(&mut self) -> () {
    self.summary.total_header_size = (self.summary.byte_size()
        + self.names.byte_size()
        + self.imports.byte_size()
        + self.exports.byte_size()
        + self.assets.byte_size()
        + self.dependencies.byte_size()) as u32;
    self.summary.name_count = self.names.names.len() as u32;
    self.summary.name_offset = self.summary.byte_size() as u32;
    self.summary.export_count = self.exports.exports.len() as u32;
    self.summary.export_offset = (self.summary.byte_size() + self.names.byte_size() + self.imports.byte_size()) as u32;
    self.summary.import_count = self.imports.objects.len() as u32;
    self.summary.import_offset = (self.summary.byte_size() + self.names.byte_size()) as u32;
    self.summary.depends_offset =
        (self.summary.byte_size() + self.names.byte_size() + self.imports.byte_size() + self.exports.byte_size())
            as u32;
    self.summary.asset_registry_data_offset =
        (self.summary.byte_size() + self.names.byte_size() + self.imports.byte_size() + self.exports.byte_size() + 4)
            as u32;
    self.summary.bulk_data_start_offset = (self.summary.byte_size()
        + self.names.byte_size()
        + self.imports.byte_size()
        + self.exports.byte_size()
        + self.assets.byte_size()
        + self.dependencies.byte_size()
        + Property::struct_size(&self.properties)) as u32;
    self.summary.preload_dependency_count = self.dependencies.dependencies.len() as u32;
    self.summary.preload_dependency_offset = (self.summary.byte_size()
        + self.names.byte_size()
        + self.imports.byte_size()
        + self.exports.byte_size()
        + self.assets.byte_size()) as u32;

    // TODO: do it for each export, not sure what that looks like though
    self.exports.exports[0].serial_size = Property::struct_size(&self.properties) as u64;
    self.exports.exports[0].serial_offset = self.summary.total_header_size;
  }
}
