use crate::file_summary::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct Name {
  pub index: u64,
  pub name: String,                  // Size as a uint32, then null terminated string
  pub non_case_preserving_hash: u16, // idk how this is calculated, idk if it matters
  pub case_preserving_hash: u16,     // ditto
}

#[derive(Debug)]
pub struct NameMap {
  pub names: Vec<Name>,
}

impl Name {
  fn read(rdr: &mut Cursor<Vec<u8>>) -> Self {
    let name = read_string(rdr);
    let non_case_preserving_hash = rdr.read_u16::<LittleEndian>().unwrap();
    let case_preserving_hash = rdr.read_u16::<LittleEndian>().unwrap();
    return Name {
      index: 0,
      name,
      non_case_preserving_hash,
      case_preserving_hash,
    };
  }
}

impl NameMap {
  pub fn read(rdr: &mut Cursor<Vec<u8>>, summary: &FileSummary) -> Result<Self, String> {
    if rdr.position() != summary.name_offset.into() {
      return Err(
        format!(
          "Error parsing NameMap: Expected to be at position {}, but I'm at position {}",
          summary.name_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let mut names = vec![];
    for i in 0..summary.name_count {
      let mut name = Name::read(rdr);
      name.index = i.into();
      names.push(name);
    }
    return Ok(NameMap { names });
  }

  pub fn byte_size(&self) -> usize {
    // Size of each is 8 + (len(name) + 1)
    let mut size = 0;
    for name in self.names.iter() {
      size += 8 + name.name.len() + 1;
    }
    return size
  }

  pub fn lookup(&self, index: u64, rep: &str) -> Result<&Name, String> {
    if index > self.names.len() as u64 {
      return Err(format!(
        "Name {} for {} is not in NameMap (length {})",
        index,
        rep,
        self.names.len()
      ));
    }
    return Ok(&self.names[index as usize]);
  }

  pub fn read_name(&self, rdr: &mut Cursor<Vec<u8>>, rep: &str) -> Result<String, String> {
    let index = rdr.read_u64::<LittleEndian>().unwrap();
    return self.lookup(index, rep).map(|x| x.name.clone());
  }
}
