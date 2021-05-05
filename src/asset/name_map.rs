use crate::asset::*;
use crate::reader::*;
use crate::util::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct Name {
  pub index: u32,
  pub name: String,                  // Size as a uint32, then null terminated string
  pub non_case_preserving_hash: u16, // idk how this is calculated, idk if it matters
  pub case_preserving_hash: u16,     // ditto
}

impl Name {
  fn read(rdr: &mut ByteReader) -> Result<Self> {
    let name = read_string(rdr)?;
    let non_case_preserving_hash = rdr.read_u16::<LittleEndian>()?;
    let case_preserving_hash = rdr.read_u16::<LittleEndian>()?;
    Ok(Name {
      index: 0,
      name,
      non_case_preserving_hash,
      case_preserving_hash,
    })
  }

  fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    write_string(curs, &self.name)?;
    curs.write_u16::<LittleEndian>(self.non_case_preserving_hash)?;
    curs.write_u16::<LittleEndian>(self.case_preserving_hash)?;
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameVariant {
  pub name: String,
  pub variant: u32,
}

impl Into<NameVariant> for String {
  fn into(self) -> NameVariant {
    NameVariant::parse(&self)
  }
}
impl Into<NameVariant> for &str {
  fn into(self) -> NameVariant {
    NameVariant::parse(self)
  }
}

impl NameVariant {
  pub fn new(name: &str, variant: u32) -> Self {
    Self {
      name: name.to_string(),
      variant,
    }
  }

  /// Parses a string to a NameVariant
  ///
  /// Strings like SomeName_6 are turned into a NameVariant with variant = 6.
  pub fn parse(txt: &str) -> Self {
    let pieces: Vec<&str> = txt.split('_').collect();
    let len = pieces.len();
    if len > 1 {
      if let Ok(variant) = pieces[len - 1].parse::<u32>() {
        let name: String = pieces[0..len - 1].join("_");
        return Self { name, variant };
      }
    }
    Self {
      name: txt.to_string(),
      variant: 0,
    }
  }

  pub fn read(rdr: &mut ByteReader, names: &NameMap) -> Result<Self> {
    let start_pos = rdr.position();
    let index = rdr.read_u32::<LittleEndian>()?;
    let name = names
      .lookup(index)
      .map(|x| x.name.clone())
      .with_context(|| format!("Failed parsing NameVariant starting at {:#X}", start_pos))?;
    let variant = read_u32(rdr)?;
    Ok(Self { name, variant })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &NameMap) -> Result<()> {
    let name_n = names
      .get_name_obj(&self.name)
      .with_context(|| format!("Name {} is not in names", self))?;
    write_u32(curs, name_n.index)?;
    write_u32(curs, self.variant)?;
    Ok(())
  }
}

impl std::fmt::Display for NameVariant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.variant != 0 {
      write!(f, "{}_{}", self.name, self.variant)
    } else {
      write!(f, "{}", self.name)
    }
  }
}

#[derive(Debug)]
pub struct NameMap {
  pub names: Vec<Name>,
}

impl NameMap {
  pub fn read(rdr: &mut ByteReader, summary: &FileSummary) -> Result<Self> {
    if rdr.position() != summary.name_offset.into() {
      bail!(
        "Wrong name map starting position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.name_offset, rdr.position()
      );
    }

    let mut names = vec![];
    for i in 0..summary.name_count {
      let start_pos = rdr.position();
      let mut name = Name::read(rdr)
        .with_context(|| format!("Failed to parse name starting at {:#X}", start_pos))?;
      name.index = i.into();
      names.push(name);
    }
    Ok(NameMap { names })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    for name in self.names.iter() {
      name.write(curs)?;
    }
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    // Size of each is 8 + (len(name) + 1)
    let mut size = 0;
    for name in self.names.iter() {
      size += 8 + name.name.len() + 1;
    }
    return size;
  }

  pub fn get_name_obj(&self, name: &str) -> Option<&Name> {
    for own_name in self.names.iter() {
      if own_name.name == name {
        return Some(own_name);
      }
    }
    return None;
  }

  pub fn lookup(&self, index: u32) -> Result<&Name> {
    if index > self.names.len() as u32 {
      bail!(
        "Name index {} is not in names (length {})",
        index,
        self.names.len()
      );
    }
    Ok(&self.names[index as usize])
  }

  pub fn add(&mut self, name: &str) -> bool {
    // No-op if name already exists
    if self.get_name_obj(name).is_some() {
      return false;
    }

    let index = self.names.len() as u32;
    let name_obj = Name {
      index,
      name: name.to_string(),
      non_case_preserving_hash: 0,
      case_preserving_hash: 0,
    };
    self.names.push(name_obj);
    return true;
  }

  pub fn read_name(&self, rdr: &mut ByteReader) -> Result<String> {
    let index = rdr.read_u32::<LittleEndian>()?;
    match self.lookup(index).map(|x| x.name.clone()) {
      Err(err) => bail!("{} at {:04X}", err, rdr.position()),
      Ok(x) => Ok(x),
    }
  }
}
