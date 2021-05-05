use crate::reader::*;
use crate::util::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct Generation {
  pub export_count: u32,
  pub name_count: u32,
}

#[derive(Debug)]
pub struct FileSummary {
  pub tag: [u8; 4],                  // 0000-0003
  pub file_version_ue4: u32,         // 0004-0007 ?
  pub file_version_license_ue4: u32, // 0008-000B
  pub custom_version: [u8; 12],      // 000C-0017

  pub total_header_size: u32, // 0018-001B

  pub package_flags: u32, // 0025-0028 ?

  // in drg this is always from "None" and from 001C-0024
  pub folder_name: String, // u32 size, then null terminated string

  pub name_count: u32,     // 0029-002C
  pub name_offset: u32,    // 002D-0030
  pub localization_id: (), // seems to always be null and take up 0 bytes

  pub gatherable_text_data_count: u32,  // 0031-0034
  pub gatherable_text_data_offset: u32, // 0035-0038

  pub export_count: u32,  // 0039-003C
  pub export_offset: u32, // 003D-0040

  pub import_count: u32,  // 0041-0044
  pub import_offset: u32, // 0045-0049

  pub depends_offset: u32, // 0049-004C

  pub soft_package_references_count: u32,
  pub soft_package_references_offset: u32,
  pub searchable_names_offset: u32,
  pub thumbnail_table_offset: u32,
  pub guid: [u8; 16],
  pub generations: Vec<Generation>,

  pub saved_by_engine_version: [u8; 16],
  pub compatible_with_engine_version: [u8; 16],
  pub compression_flags: u32,
  pub package_source: i64,

  pub asset_registry_data_offset: u32,  // 00A5-00A8
  pub bulk_data_start_offset: u32,      // 00A9-00AC
  pub world_tile_info_data_offset: u32, // 00AD-00B0
  pub chunk_ids: u64, // this should be an array of some structs, idk what they are though
  pub preload_dependency_count: u32, // 00B9-00BC
  pub preload_dependency_offset: u32, // 00BD-00CA
}

impl Generation {
  fn read(rdr: &mut ByteReader) -> Result<Self> {
    let export_count = read_u32(rdr)?;
    let name_count = read_u32(rdr)?;
    Ok(Generation {
      export_count,
      name_count,
    })
  }

  fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    write_u32(curs, self.export_count)?;
    write_u32(curs, self.name_count)?;
    Ok(())
  }

  fn read_array(rdr: &mut ByteReader) -> Result<Vec<Generation>> {
    let length = read_u32(rdr)?;
    let mut generations: Vec<Generation> = vec![];
    for _ in 0..length {
      generations.push(Self::read(rdr)?);
    }
    Ok(generations)
  }

  fn write_array(gens: &Vec<Generation>, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    write_u32(curs, gens.len() as u32)?;
    for i in 0..gens.len() {
      gens[i].write(curs)?;
    }
    Ok(())
  }
}

impl FileSummary {
  pub fn read(rdr: &mut ByteReader) -> Result<Self> {
    let tag: [u8; 4] = read_bytes(rdr, 4)?;
    let file_version_ue4 = read_u32(rdr)?;
    let file_version_license_ue4 = read_u32(rdr)?;
    let custom_version: [u8; 12] = read_bytes(rdr, 12)?;
    let total_header_size = read_u32(rdr)?;
    let folder_name = read_string(rdr)?;
    let package_flags = read_u32(rdr)?;
    let name_count = read_u32(rdr)?;
    let name_offset = read_u32(rdr)?;
    let gatherable_text_data_count = read_u32(rdr)?;
    let gatherable_text_data_offset = read_u32(rdr)?;
    let export_count = read_u32(rdr)?;
    let export_offset = read_u32(rdr)?;
    let import_count = read_u32(rdr)?;
    let import_offset = read_u32(rdr)?;
    let depends_offset = read_u32(rdr)?;
    let soft_package_references_count = read_u32(rdr)?;
    let soft_package_references_offset = read_u32(rdr)?;
    let searchable_names_offset = read_u32(rdr)?;
    let thumbnail_table_offset = read_u32(rdr)?;
    let guid: [u8; 16] = read_bytes(rdr, 16)?;
    let generations = Generation::read_array(rdr)?;
    let saved_by_engine_version: [u8; 16] = read_bytes(rdr, 16)?;
    let compatible_with_engine_version: [u8; 16] = read_bytes(rdr, 16)?;
    let compression_flags = read_u32(rdr)?;
    let package_source = rdr.read_i64::<LittleEndian>()?;
    let asset_registry_data_offset = read_u32(rdr)?;
    let bulk_data_start_offset = read_u32(rdr)?;
    let world_tile_info_data_offset = read_u32(rdr)?;
    let chunk_ids = rdr.read_u64::<LittleEndian>()?;
    let preload_dependency_count = read_u32(rdr)?;
    let preload_dependency_offset = read_u32(rdr)?;

    Ok(FileSummary {
      tag,
      file_version_ue4,
      file_version_license_ue4,
      custom_version,
      total_header_size,
      package_flags,
      folder_name,
      name_count,
      name_offset,
      localization_id: (),
      gatherable_text_data_count,
      gatherable_text_data_offset,
      export_count,
      export_offset,
      import_count,
      import_offset,
      depends_offset,
      soft_package_references_count,
      soft_package_references_offset,
      searchable_names_offset,
      thumbnail_table_offset,
      guid,
      generations,
      saved_by_engine_version,
      compatible_with_engine_version,
      compression_flags,
      package_source,
      asset_registry_data_offset,
      bulk_data_start_offset,
      world_tile_info_data_offset,
      chunk_ids,
      preload_dependency_count,
      preload_dependency_offset,
    })
  }

  pub fn write(&self, curs: &mut Cursor<Vec<u8>>) -> Result<()> {
    curs.write(&self.tag)?;
    write_u32(curs, self.file_version_ue4)?;
    write_u32(curs, self.file_version_license_ue4)?;
    curs.write(&self.custom_version)?;
    write_u32(curs, self.total_header_size)?;
    write_string(curs, &self.folder_name)?;
    write_u32(curs, self.package_flags)?;
    write_u32(curs, self.name_count)?;
    write_u32(curs, self.name_offset)?;
    write_u32(curs, self.gatherable_text_data_count)?;
    write_u32(curs, self.gatherable_text_data_offset)?;
    write_u32(curs, self.export_count)?;
    write_u32(curs, self.export_offset)?;
    write_u32(curs, self.import_count)?;
    write_u32(curs, self.import_offset)?;
    write_u32(curs, self.depends_offset)?;
    write_u32(curs, self.soft_package_references_count)?;
    write_u32(curs, self.soft_package_references_offset)?;
    write_u32(curs, self.searchable_names_offset)?;
    write_u32(curs, self.thumbnail_table_offset)?;
    curs.write(&self.guid)?;
    Generation::write_array(&self.generations, curs)?;
    curs.write(&self.saved_by_engine_version)?;
    curs.write(&self.compatible_with_engine_version)?;
    write_u32(curs, self.compression_flags)?;
    curs.write_i64::<LittleEndian>(self.package_source)?;
    write_u32(curs, self.asset_registry_data_offset)?;
    write_u32(curs, self.bulk_data_start_offset)?;
    write_u32(curs, self.world_tile_info_data_offset)?;
    curs.write_u64::<LittleEndian>(self.chunk_ids)?;
    write_u32(curs, self.preload_dependency_count)?;
    write_u32(curs, self.preload_dependency_offset)?;
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    // Size is 188 + (len(folder_name) + 1)
    188 + self.folder_name.len() + 1
  }
}
