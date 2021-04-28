use crate::asset::*;
use crate::util::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq)]
pub enum Dependency {
  UObject,
  Import(NameVariant),
  Export(NameVariant),
}

impl Dependency {
  pub fn import<T: Into<NameVariant>>(name: T) -> Self {
    Self::Import(name.into())
  }
  pub fn export<T: Into<NameVariant>>(name: T) -> Self {
    Self::Export(name.into())
  }
  pub fn uobject() -> Self {
    Self::UObject
  }

  pub fn serialize(&self, imports: &ObjectImports, exports: &ObjectExports) -> i32 {
    match self {
      Self::UObject => 0,
      Self::Import(name) => imports
        .index_of(name)
        .expect("Invalid Dependency::Import name"),
      Self::Export(name) => exports
        .serialized_index_of(name)
        .expect("Invalid Dependency::Export export name") as i32,
    }
  }

  pub fn deserialize(idx: i32, imports: &ObjectImports, exports: &ObjectExports) -> Result<Self> {
    if idx == 0 {
      Ok(Self::UObject)
    } else if idx < 0 {
      let import = imports.lookup((-idx - 1) as u64)?;
      Ok(Self::Import(import.name.clone()))
    } else {
      let export = exports.lookup((idx - 1) as u64)?;
      Ok(Self::Export(export.object_name.clone()))
    }
  }

  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<Self> {
    let idx = rdr.read_i32::<LittleEndian>()?;
    Self::deserialize(idx, imports, exports)
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<()> {
    let dep_i = match self {
      Self::UObject => 0,
      Self::Import(name) => imports
        .serialized_index_of(name)
        .with_context(|| format!("Name {} is not imported", name))?,
      Self::Export(name) => exports
        .serialized_index_of(name)
        .with_context(|| format!("Name {} is not exported", name))?,
    };
    write_u32(curs, dep_i)?;
    Ok(())
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
  ) -> Result<Self> {
    if rdr.position() != summary.preload_dependency_offset.into() {
      bail!(
        "Wrong exports dependency position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.preload_dependency_offset,
        rdr.position(),
      );
    }

    let mut dependencies = vec![];
    for _ in 0..summary.preload_dependency_count {
      let start_pos = rdr.position();
      let dependency = Dependency::read(rdr, imports, exports)
        .with_context(|| format!("Failed to parse dependency starting at {:#X}", start_pos))?;
      dependencies.push(dependency);
    }

    return Ok(PreloadDependencies { dependencies });
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    imports: &ObjectImports,
    exports: &ObjectExports,
  ) -> Result<()> {
    for dep in self.dependencies.iter() {
      dep.write(curs, imports, exports)?;
    }
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    // 4 bytes per string index
    self.dependencies.len() * 4
  }

  // TODO check for duplicates
  pub fn add_import(&mut self, name: &str) -> () {
    self.dependencies.push(Dependency::import(name));
  }
  pub fn add_export(&mut self, name: &str) -> () {
    self.dependencies.push(Dependency::export(name));
  }
}
