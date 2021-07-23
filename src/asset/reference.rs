use crate::asset::*;
use crate::reader::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::cmp::Ordering;
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
  UObject,
  Import(NameVariant),
  Export(NameVariant),
}

impl Reference {
  pub fn import(name: NameVariant) -> Self {
    Self::Import(name)
  }
  pub fn export(name: NameVariant) -> Self {
    Self::Export(name)
  }
  pub fn uobject() -> Self {
    Self::UObject
  }

  pub fn serialize(&self, imports: &Imports, exports: &Exports) -> i32 {
    match self {
      Self::UObject => 0,
      Self::Import(name) => imports
        .index_of(name)
        .expect("Invalid Reference::Import name"),
      Self::Export(name) => exports
        .serialized_index_of(name)
        .expect("Invalid Reference::Export export name") as i32,
    }
  }

  pub fn deserialize(idx: i32, imports: &Imports, exports: &Exports) -> Result<Self> {
    match idx.cmp(&0) {
      Ordering::Equal => Ok(Self::UObject),
      Ordering::Less => {
        let import = imports.lookup((-idx - 1) as u64)?;
        Ok(Self::Import(import.name.clone()))
      }
      Ordering::Greater => {
        let export = exports.lookup((idx - 1) as u64)?;
        Ok(Self::Export(export.object_name.clone()))
      }
    }
  }

  pub fn read(rdr: &mut ByteReader, imports: &Imports, exports: &Exports) -> Result<Self> {
    let idx = rdr.read_i32::<LittleEndian>()?;
    Self::deserialize(idx, imports, exports)
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    names: &Names,
    imports: &Imports,
    exports: &Exports,
  ) -> Result<()> {
    let dep_i = match self {
      Self::UObject => 0,
      Self::Import(name) => imports
        .serialized_index_of(name)
        .with_context(|| format!("Name {} is not imported", name.to_string(names)))?,
      Self::Export(name) => exports
        .serialized_index_of(name)
        .with_context(|| format!("Name {} is not exported", name.to_string(names)))?,
    };
    write_u32(curs, dep_i)?;
    Ok(())
  }

  pub fn to_string(&self, names: &Names) -> String {
    match self {
      Self::UObject => "UObject".to_string(),
      Self::Import(name) => format!("Import {}", name.to_string(names)),
      Self::Export(name) => format!("Export {}", name.to_string(names)),
    }
  }
}
