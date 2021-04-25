use crate::file_summary::*;
use crate::name_map::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct ObjectImport {
  class_package: String, // stored in file as uint64 index into name_map
  class: String,         // same as before
  outer_index: i32,      // idk what this represents
  name: String,          // same as class_package
}

#[derive(Debug)]
pub struct ObjectImports {
  objects: Vec<ObjectImport>,
}

impl ObjectImport {
  fn read(rdr: &mut Cursor<Vec<u8>>, name_map: &NameMap) -> Result<Self, String> {
    let class_package = name_map.read_name(rdr, "class_package")?;
    let class = name_map.read_name(rdr, "class")?;
    let outer_index = rdr.read_i32::<LittleEndian>().unwrap();
    let name = name_map.read_name(rdr, "name")?;
    return Ok(ObjectImport {
      class_package,
      class,
      outer_index,
      name,
    });
  }
}

impl ObjectImports {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    summary: &FileSummary,
    name_map: &NameMap,
  ) -> Result<Self, String> {
    if rdr.position() != summary.import_offset.into() {
      return Err(
        format!(
          "Error parsing ObjectImports: Expected to be at position {}, but I'm at position {}",
          summary.import_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let mut objects = vec![];
    for _ in 0..summary.import_count {
      let object = ObjectImport::read(rdr, name_map)?;
      objects.push(object);
    }
    return Ok(ObjectImports { objects });
  }

  pub fn byte_size(&self) -> usize {
    // Each ObjectImport is 28 bytes long
    28 * self.objects.len()
  }

  pub fn lookup(&self, index: u64, rep: &str) -> Result<&ObjectImport, String> {
    if index > self.objects.len() as u64 {
      return Err(format!(
        "Import {} for {} is not in ObjectImports (length {})",
        index,
        rep,
        self.objects.len()
      ));
    }
    return Ok(&self.objects[index as usize]);
  }

  pub fn read_import(&self, rdr: &mut Cursor<Vec<u8>>, rep: &str) -> Result<String, String> {
    let index_raw = rdr.read_u32::<LittleEndian>().unwrap();
    let index = std::u32::MAX - index_raw; // import indices are stored as -index - 1, for some reason
    return self.lookup(index.into(), rep).map(|x| x.name.clone());
  }
}
