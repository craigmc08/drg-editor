use crate::asset::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct ObjectImport {
  pub class_package: String, // stored in file as uint64 index into name_map
  pub class: String,         // same as before
  pub outer_index: i32,      // idk what this represents
  pub name: String,          // same as class_package
}

#[derive(Debug)]
pub struct ObjectImports {
  pub objects: Vec<ObjectImport>,
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

  fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> () {
    let cpkg = names
      .get_name_obj(&self.class_package)
      .expect("Invalid ObjectImport class_package");
    let class = names
      .get_name_obj(&self.class)
      .expect("Invalid ObjectImport class");
    let name = names
      .get_name_obj(&self.name)
      .expect("Invalid ObjectImport name");
    curs.write_u64::<LittleEndian>(cpkg.index).unwrap();
    curs.write_u64::<LittleEndian>(class.index).unwrap();
    curs.write_i32::<LittleEndian>(self.outer_index).unwrap();
    curs.write_u64::<LittleEndian>(name.index).unwrap();
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

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) {
    for object in self.objects.iter() {
      object.write(curs, names);
    }
  }

  pub fn byte_size(&self) -> usize {
    // Each ObjectImport is 28 bytes long
    28 * self.objects.len()
  }

  pub fn serialized_index_of(&self, object: &str) -> Option<u32> {
    let mut i: u32 = 0;
    for import in self.objects.iter() {
      if import.name == object {
        return Some(std::u32::MAX - i);
      }
      i += 1;
    }
    return None
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
