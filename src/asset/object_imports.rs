use crate::asset::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct ObjectImport {
  pub class_package: NameVariant,
  pub class: NameVariant,
  pub outer_index: i32, // idk what this represents
  pub name: NameVariant,
}

#[derive(Debug)]
pub struct ObjectImports {
  pub objects: Vec<ObjectImport>,
}

impl ObjectImport {
  fn read(rdr: &mut Cursor<Vec<u8>>, name_map: &NameMap) -> Result<Self> {
    let class_package = NameVariant::read(rdr, name_map).with_context(|| "class_package")?;
    let class = NameVariant::read(rdr, name_map).with_context(|| "class")?;
    let outer_index = rdr.read_i32::<LittleEndian>()?;
    let name = NameVariant::read(rdr, name_map).with_context(|| "name")?;
    return Ok(ObjectImport {
      class_package,
      class,
      outer_index,
      name,
    });
  }

  fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<()> {
    self
      .class_package
      .write(curs, names)
      .with_context(|| "class_package")?;
    self.class.write(curs, names).with_context(|| "class")?;
    curs.write_i32::<LittleEndian>(self.outer_index)?;
    self.name.write(curs, names).with_context(|| "name")?;
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

  pub fn index_of(&self, object: &NameVariant) -> Option<i32> {
    let mut i: i32 = 0;
    for import in self.objects.iter() {
      if import.name == *object {
        return Some(-i - 1);
      }
      i += 1;
    }
    return None;
  }

  pub fn serialized_index_of(&self, object: &NameVariant) -> Option<u32> {
    self.index_of(object).map(|i| i as u32)
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

  pub fn add(
    &mut self,
    class_package: NameVariant,
    class: NameVariant,
    name: NameVariant,
    outer_index: i32,
  ) -> i32 {
    if let Some(index) = self.serialized_index_of(&name) {
      // No-op if the object is already imported
      // TODO what to do if different class_package/class/outer_index?
      return -(index as i32) - 1;
    }

    let object = ObjectImport {
      class_package,
      class,
      outer_index,
      name,
    };
    let len = self.objects.len();
    self.objects.push(object);
    return -(len as i32) - 1;
  }

  pub fn read_import(&self, rdr: &mut Cursor<Vec<u8>>) -> Result<NameVariant> {
    let index_raw = rdr.read_u32::<LittleEndian>()?;
    let index = std::u32::MAX - index_raw; // import indices are stored as -index - 1, for some reason
    Ok(self.lookup(index.into()).map(|x| x.name.clone())?)
  }
}
