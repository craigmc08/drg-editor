use crate::asset::*;
use crate::reader::*;
use crate::util::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct Export {
  pub class: u32,       // just storing u32 because idc what this is
  pub super_index: i32, // not sure what this represents
  pub template: u32,    // just storing u32 because idc what this is
  pub outer: i32,       // again, idk
  pub object_name: NameVariant,
  pub object_flags: u32,       // just some bytes
  pub serial_size: u64, // size of uexp struct, 4 bytes of padding after this are incorporated into the value?
  pub serial_offset: u32, // same as file_summary.total_header_size
  pub export_file_offset: u64, // NOT STORED IN FILE
  pub forced_export: bool, // 4 bytes
  pub not_for_client: bool, // 4 bytes
  pub not_for_server: bool, // 4 bytes
  pub was_filtered: bool, // 4 bytes
  pub package_guid: [u8; 16],
  pub package_flags: u32,
  pub not_always_loaded_for_editor_game: bool, // 4 bytes
  pub is_asset: bool,                          // 4 bytes
  pub first_export_dependency: u32,
  pub serialization_before_serialization_dependencies: u32,
  pub create_before_serialization_dependencies: u32,
  pub serialization_before_create_dependencies: u32,
  pub create_before_create_dependencies: u32,
}

#[derive(Debug)]
pub struct Exports {
  pub exports: Vec<Export>,
}

impl Export {
  fn read(rdr: &mut ByteReader, names: &Names) -> Result<Self> {
    let class = read_u32(rdr)?;
    let super_index = rdr.read_i32::<LittleEndian>()?;
    let template = read_u32(rdr)?;
    let outer = rdr.read_i32::<LittleEndian>()?;
    let object_name = NameVariant::read(rdr, names).with_context(|| "object_name")?;
    let object_flags = rdr.read_u32::<LittleEndian>()?;
    let serial_size = rdr.read_u64::<LittleEndian>()?;
    let serial_offset = read_u32(rdr)?;
    let forced_export = read_bool(rdr)?;
    let not_for_client = read_bool(rdr)?;
    let not_for_server = read_bool(rdr)?;
    let was_filtered = read_bool(rdr)?;
    let package_guid: [u8; 16] = read_bytes(rdr, 16)?;
    let package_flags = read_u32(rdr)?;
    let not_always_loaded_for_editor_game = read_bool(rdr)?;
    let is_asset = read_bool(rdr)?;
    let first_export_dependency = read_u32(rdr)?;
    let serialization_before_serialization_dependencies = read_u32(rdr)?;
    let create_before_serialization_dependencies = read_u32(rdr)?;
    let serialization_before_create_dependencies = read_u32(rdr)?;
    let create_before_create_dependencies = read_u32(rdr)?;
    Ok(Export {
      class,
      super_index,
      template,
      outer,
      object_name,
      object_flags,
      serial_size,
      serial_offset,
      export_file_offset: 0,
      forced_export,
      not_for_client,
      not_for_server,
      was_filtered,
      package_guid,
      package_flags,
      not_always_loaded_for_editor_game,
      is_asset,
      first_export_dependency,
      serialization_before_serialization_dependencies,
      create_before_serialization_dependencies,
      serialization_before_create_dependencies,
      create_before_create_dependencies,
    })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &Names) -> Result<()> {
    write_u32(curs, self.class)?;
    curs.write_i32::<LittleEndian>(self.super_index)?;
    write_u32(curs, self.template)?;
    curs.write_i32::<LittleEndian>(self.outer)?;
    self
      .object_name
      .write(curs, names)
      .with_context(|| "object_name")?;
    curs.write_u32::<LittleEndian>(self.object_flags)?;
    curs.write_u64::<LittleEndian>(self.serial_size)?;
    write_u32(curs, self.serial_offset)?;
    write_bool(curs, self.forced_export)?;
    write_bool(curs, self.not_for_client)?;
    write_bool(curs, self.not_for_server)?;
    write_bool(curs, self.was_filtered)?;
    curs.write_all(&self.package_guid)?;
    write_u32(curs, self.package_flags)?;
    write_bool(curs, self.not_always_loaded_for_editor_game)?;
    write_bool(curs, self.is_asset)?;
    write_u32(curs, self.first_export_dependency)?;
    write_u32(curs, self.serialization_before_serialization_dependencies)?;
    write_u32(curs, self.create_before_serialization_dependencies)?;
    write_u32(curs, self.serialization_before_create_dependencies)?;
    write_u32(curs, self.create_before_create_dependencies)?;
    Ok(())
  }
}

impl Exports {
  pub fn read(rdr: &mut ByteReader, summary: &FileSummary, names: &Names) -> Result<Self> {
    if rdr.position() != summary.export_offset as u64 {
      bail!(
        "Wrong exports starting position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.export_offset,
        rdr.position(),
      );
    }

    let mut exports = vec![];
    let mut export_file_offset = 0;
    for _ in 0..summary.export_count {
      let start_pos = rdr.position();
      let mut object = Export::read(rdr, names)
        .with_context(|| format!("Failed to parse export starting at {:#X}", start_pos))?;

      // Compute export_file_offset based on the size of preceeding exports
      object.export_file_offset = export_file_offset;
      export_file_offset += object.serial_size;

      exports.push(object);
    }
    Ok(Exports { exports })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>, names: &Names) -> Result<()> {
    for export in self.exports.iter() {
      export.write(curs, names)?;
    }
    Ok(())
  }

  pub fn serialized_index_of(&self, object: &NameVariant) -> Option<u32> {
    for (i, export) in self.exports.iter().enumerate() {
      if export.object_name == *object {
        return Some(i as u32 + 1);
      }
    }
    None
  }

  pub fn lookup(&self, index: u64) -> Result<&Export> {
    if index > self.exports.len() as u64 {
      bail!(
        "Export index {} is not in exports (length {})",
        index,
        self.exports.len()
      );
    }
    Ok(&self.exports[index as usize])
  }

  pub fn byte_size(&self) -> usize {
    // Always 104 bytes long
    104
  }
}
