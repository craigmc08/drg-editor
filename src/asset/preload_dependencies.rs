use crate::asset::*;
use crate::util::*;
use std::io::Cursor;

#[derive(Debug)]
pub enum Dependency {
  UObject,
  Import(String, u32),
  Export(String, u32),
}

impl Dependency {
  pub fn read(rdr: &mut Cursor<Vec<u8>>, imports: &ObjectImports, exports: &ObjectExports) -> Result<Self, String> {
    let idx = read_u32(rdr);
    // If idx (as an i32) is negative
    if idx == 0 {
      Ok(Self::UObject)
    } else if (idx & 0x80000000) > 0 {
      let import = imports.lookup(
        (std::u32::MAX - idx) as u64,
        &format!("PreloadDependency import @ {:04X}", rdr.position()),
      )?;
      Ok(Self::Import(import.name.clone(), import.name_variant))
    } else {
      let export = exports.lookup(
        (idx - 1) as u64,
        &format!("PreloadDependency export @ {:04X}", rdr.position()),
      )?;
      Ok(Self::Export(export.object_name.clone(), export.object_name_variant))
    }
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, imports: &ObjectImports, exports: &ObjectExports) -> () {
    let dep_i = match self {
      Self::UObject => 0,
      Self::Import(name, variant) => imports
        .serialized_index_of(name, *variant)
        .expect("Invalid PreloadDependency import name"),
      Self::Export(name, variant) => exports
        .serialized_index_of(name, *variant)
        .expect("Invalid PreloadDependency export name"),
    };
    write_u32(curs, dep_i);
  }
}

#[derive(Debug)]
pub struct PreloadDependencies {
  // each n in the dependencies array is:
  // - negative? then -n - 1 is the index into imports
  // - positive? then n - 1 is the index into exports
  pub dependencies: Vec<Dependency>,
}

impl PreloadDependencies {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    summary: &FileSummary,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<Self, String> {
    if rdr.position() != summary.preload_dependency_offset.into() {
      return Err(
        format!(
          "Error parsing PreloadDependencies: Expected to be at position {:04X}, but I'm at position {:04X}",
          summary.preload_dependency_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let mut dependencies = vec![];
    for _ in 0..summary.preload_dependency_count {
      let dependency = Dependency::read(rdr, imports, exports)?;
      dependencies.push(dependency);
      // let idx = read_u32(rdr);
      // // If idx (as an i32) is negative
      // if (idx & 0x80000000) > 0 {
      //   let import = imports.lookup(
      //     (std::u32::MAX - idx) as u64,
      //     &format!("PreloadDependency import @ {:04X}", rdr.position()),
      //   )?;
      //   dependencies.push(Dependency::Import(import.name.clone(), import.name_variant));
      // } else {
      //   let export = exports.lookup(
      //     (idx - 1) as u64,
      //     &format!("PreloadDependency export @ {:04X}", rdr.position()),
      //   )?;
      //   dependencies.push(Dependency::Export(export.object_name.clone(), export.object_name_variant));
      // }
    }

    return Ok(PreloadDependencies { dependencies });
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> () {
    for dep in self.dependencies.iter() {
      dep.write(curs, imports, exports);
      // let dep_i = match dep {
      //   Dependency::Import(name, variant) => imports
      //     .serialized_index_of(name, *variant)
      //     .expect("Invalid PreloadDependency import name"),
      //   Dependency::Export(name, variant) => exports
      //     .serialized_index_of(name, *variant)
      //     .expect("Invalid PreloadDependency export name"),
      // };
      // write_u32(curs, dep_i);
    }
  }

  pub fn byte_size(&self) -> usize {
    // 4 bytes per string index
    self.dependencies.len() * 4
  }

  // TODO check for duplicates
  pub fn add_import(&mut self, name: &str) -> () {
    self.dependencies.push(Dependency::Import(name.to_string(), 0));
  }
  pub fn add_export(&mut self, name: &str) -> () {
    self.dependencies.push(Dependency::Export(name.to_string(), 0));
  }
}
