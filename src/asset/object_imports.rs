use crate::asset::*;
use crate::util::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct ObjectImport {
  pub class_package: String, // stored in file as uint64 index into name_map
  pub cpkg_variant: u32,
  pub class: String, // same as before
  pub class_variant: u32,
  pub outer_index: i32, // idk what this represents
  pub name: String,     // same as class_package
  pub name_variant: u32,
}

#[derive(Debug)]
pub struct ObjectImports {
  pub objects: Vec<ObjectImport>,
}

impl ObjectImport {
  fn read(rdr: &mut Cursor<Vec<u8>>, name_map: &NameMap) -> Result<Self> {
    let (class_package, cpkg_variant) = name_map
      .read_name_with_variant(rdr)
      .with_context(|| "For class_package")?;
    let (class, class_variant) = name_map
      .read_name_with_variant(rdr)
      .with_context(|| "For class")?;
    let outer_index = rdr.read_i32::<LittleEndian>()?;
    let (name, name_variant) = name_map
      .read_name_with_variant(rdr)
      .with_context(|| "For name")?;
    return Ok(ObjectImport {
      class_package,
      cpkg_variant,
      class,
      class_variant,
      outer_index,
      name,
      name_variant,
    });
  }

  fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<()> {
    let cpkg = names.get_name_obj(&self.class_package).with_context(|| {
      format!(
        "ObjectImport.class_package {} is not in names",
        self.class_package
      )
    })?;
    let class = names
      .get_name_obj(&self.class)
      .with_context(|| format!("ObjectImport.class {} is not in names", self.class))?;
    let name = names
      .get_name_obj(&self.name)
      .with_context(|| format!("ObjectImport.name {} is not in names", self.name))?;
    curs.write_u32::<LittleEndian>(cpkg.index)?;
    write_u32(curs, self.cpkg_variant)?;
    curs.write_u32::<LittleEndian>(class.index)?;
    write_u32(curs, self.class_variant)?;
    curs.write_i32::<LittleEndian>(self.outer_index)?;
    curs.write_u32::<LittleEndian>(name.index)?;
    write_u32(curs, self.name_variant)?;
    Ok(())
  }
}

impl ObjectImports {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    summary: &FileSummary,
    name_map: &NameMap,
  ) -> Result<Self> {
    if rdr.position() != summary.import_offset.into() {
      bail!(
        "Wrong imports starting position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.import_offset,
        rdr.position(),
      );
    }

    let mut objects = vec![];
    for _ in 0..summary.import_count {
      let start_pos = rdr.position();
      let object = ObjectImport::read(rdr, name_map)
        .with_context(|| format!("Failed to parse import starting at {:#X}", start_pos))?;
      objects.push(object);
    }
    return Ok(ObjectImports { objects });
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<()> {
    for object in self.objects.iter() {
      object.write(curs, names)?;
    }
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    // Each ObjectImport is 28 bytes long
    28 * self.objects.len()
  }

  pub fn index_of(&self, object: &str, variant: u32) -> Option<i32> {
    let mut i: i32 = 0;
    for import in self.objects.iter() {
      if import.name == object && import.name_variant == variant {
        return Some(-i - 1);
      }
      i += 1;
    }
    return None;
  }

  pub fn serialized_index_of(&self, object: &str, variant: u32) -> Option<u32> {
    self.index_of(object, variant).map(|i| i as u32)
  }

  pub fn lookup(&self, index: u64) -> Result<&ObjectImport> {
    if index > self.objects.len() as u64 {
      bail!(
        "Import index {} is not in imports (length {})",
        index,
        self.objects.len()
      );
    }
    return Ok(&self.objects[index as usize]);
  }

  pub fn add(&mut self, class_package: &str, class: &str, name: &str, idx: i32) -> Option<i32> {
    if self.serialized_index_of(name, 0).is_some() {
      // No-op if the object is already imported
      // TODO what to do if different class_package/class/idx?
      return None;
    }

    let object = ObjectImport {
      class_package: class_package.to_string(),
      cpkg_variant: 0,
      class: class.to_string(),
      class_variant: 0,
      name: name.to_string(),
      name_variant: 0,
      outer_index: idx,
    };
    let len = self.objects.len();
    self.objects.push(object);
    return Some(-(len as i32) - 1);
  }

  pub fn read_import(&self, rdr: &mut Cursor<Vec<u8>>) -> Result<String> {
    let index_raw = rdr.read_u32::<LittleEndian>()?;
    let index = std::u32::MAX - index_raw; // import indices are stored as -index - 1, for some reason
    Ok(self.lookup(index.into()).map(|x| x.name.clone())?)
  }
}
