use crate::file_summary::*;
use crate::name_map::*;
use crate::object_imports::*;
use crate::util::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryInto;
use std::io::Cursor;

#[derive(Debug)]
pub struct ObjectExport {
  pub class: String, // stored as (-n - 1) int32 index into object_imports, or uint32_max - value
  pub super_index: i32, // not sure what this represents
  pub template: String, // stored same as `class`
  pub outer: i32,    // again, idk
  pub object_name: String, // stored as index into name_map
  pub object_flags: u64, // just some bytes
  pub serial_size: u64, // size of uexp struct, 4 bytes of padding after this are incorporated into the value?
  pub serial_offset: u32, // same as file_summary.total_header_size
  pub export_file_offset: u32,
  pub forced_export: bool,  // 4 bytes
  pub not_for_client: bool, // 4 bytes
  pub not_for_server: bool, // 4 bytes
  pub was_filtered: bool,   // 4 bytes
  pub package_guid: [u8; 16],
  pub package_flags: u32,                      // doesn't exist in file?
  pub not_always_loaded_for_editor_game: bool, // 4 bytes
  pub is_asset: bool,                          // 4 bytes
  pub first_export_dependency: u32,
  pub serialization_before_serialization_dependencies: u32,
  pub create_before_serialization_dependencies: u32,
  pub serialization_before_create_dependencies: u32,
  pub create_before_create_dependencies: u32,
}

#[derive(Debug)]
pub struct ObjectExports {
  pub exports: Vec<ObjectExport>,
}

impl ObjectExport {
  fn read(
    rdr: &mut Cursor<Vec<u8>>,
    names: &NameMap,
    imports: &ObjectImports,
  ) -> Result<Self, String> {
    let class = imports.read_import(rdr, "class")?;
    let super_index = rdr.read_i32::<LittleEndian>().unwrap();
    let template = imports.read_import(rdr, "template")?;
    let outer = rdr.read_i32::<LittleEndian>().unwrap();
    let object_name = names.lookup(read_u32(rdr).into(), "object_name").map(|x| x.name.clone())?;
    let object_flags = rdr.read_u64::<LittleEndian>().unwrap();
    let serial_size = rdr.read_u64::<LittleEndian>().unwrap();
    let serial_offset = read_u32(rdr);
    let export_file_offset = read_u32(rdr);
    let forced_export = read_bool(rdr);
    let not_for_client = read_bool(rdr);
    let not_for_server = read_bool(rdr);
    let was_filtered = read_bool(rdr);
    let package_guid = read_bytes(rdr, 16);
    let not_always_loaded_for_editor_game = read_bool(rdr);
    let is_asset = read_bool(rdr);
    let first_export_dependency = read_u32(rdr);
    let serialization_before_serialization_dependencies = read_u32(rdr);
    let create_before_serialization_dependencies = read_u32(rdr);
    let serialization_before_create_dependencies = read_u32(rdr);
    let create_before_create_dependencies = read_u32(rdr);
    return Ok(ObjectExport {
      class,
      super_index,
      template,
      outer,
      object_name,
      object_flags,
      serial_size,
      serial_offset,
      export_file_offset,
      forced_export,
      not_for_client,
      not_for_server,
      was_filtered,
      package_guid: package_guid[0..16].try_into().unwrap(),
      package_flags: 0,
      not_always_loaded_for_editor_game,
      is_asset,
      first_export_dependency,
      serialization_before_serialization_dependencies,
      create_before_serialization_dependencies,
      serialization_before_create_dependencies,
      create_before_create_dependencies,
    });
  }
}

impl ObjectExports {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    summary: &FileSummary,
    names: &NameMap,
    imports: &ObjectImports,
  ) -> Result<Self, String> {
    if rdr.position() != summary.export_offset.into() {
      return Err(
        format!(
          "Error parsing ObjectExports: Expected to be at position {}, but I'm at position {}",
          summary.export_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let mut exports = vec![];
    for _ in 0..summary.export_count {
      let object = ObjectExport::read(rdr, names, imports)?;
      exports.push(object);
    }
    return Ok(ObjectExports { exports });
  }

  pub fn byte_size(&self) -> usize {
    // Always 104 bytes long
    104
  }
}
