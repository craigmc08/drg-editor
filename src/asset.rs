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
use std::io::prelude::Write;
use crate::util::*;

pub struct Asset {
  pub summary: FileSummary,
  pub names: NameMap,
  pub imports: ObjectImports,
  pub exports: ObjectExports,
  pub assets: AssetRegistry,
  pub dependencies: PreloadDependencies,
  pub structs: Vec<Struct>,
}

impl Asset {
  pub fn read(uasset: Vec<u8>, uexp: Vec<u8>) -> Self {
    let mut cursor_uasset = Cursor::new(uasset);
    let summary = FileSummary::read(&mut cursor_uasset);
    let names = NameMap::read(&mut cursor_uasset, &summary).unwrap();
    let imports = ObjectImports::read(&mut cursor_uasset, &summary, &names).unwrap();
    let exports = ObjectExports::read(&mut cursor_uasset, &summary, &names, &imports).unwrap();
    let assets = AssetRegistry::read(&mut cursor_uasset, &summary).unwrap();
    let dependencies = PreloadDependencies::read(&mut cursor_uasset, &summary, &imports, &exports).unwrap();

    let mut cursor_uexp = Cursor::new(uexp);
    // Read all export structs
    let mut structs = vec!();
    for export in exports.exports.iter() {
      let strct = Struct::read(&mut cursor_uexp, export, &names, &imports, &exports).unwrap();
      structs.push(strct);
    }

    return Asset {
      summary,
      names,
      imports,
      exports,
      assets,
      dependencies,
      structs,
    }
  }

  pub fn write(&self) -> (Vec<u8>, Vec<u8>) {
    let mut cursor_uasset = Cursor::new(vec!());
    self.summary.write(&mut cursor_uasset);
    self.names.write(&mut cursor_uasset);
    self.imports.write(&mut cursor_uasset, &self.names);
    self.exports.write(&mut cursor_uasset, &self.names, &self.imports);
    self.assets.write(&mut cursor_uasset);
    self.dependencies.write(&mut cursor_uasset, &self.imports, &self.exports);

    let mut cursor_uexp = Cursor::new(vec!());
    for strct in self.structs.iter() {
      strct.write(&mut cursor_uexp, &self.names, &self.imports, &self.exports);
    }
    cursor_uexp.write(&self.summary.tag);

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
        + Struct::total_size(&self.structs)) as u32;
    self.summary.preload_dependency_count = self.dependencies.dependencies.len() as u32;
    self.summary.preload_dependency_offset = (self.summary.byte_size()
        + self.names.byte_size()
        + self.imports.byte_size()
        + self.exports.byte_size()
        + self.assets.byte_size()) as u32;

    let mut export_offset = self.summary.total_header_size;
    for i in 0..(self.summary.export_count as usize) {
      self.exports.exports[i].serial_size =  Struct::total_size(&self.structs) as u64;
      export_offset += self.exports.exports[i].serial_size as u32;
      self.exports.exports[i].serial_offset = export_offset;
    }
  }
}
