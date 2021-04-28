use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryFrom;
use std::io::prelude::*;
use std::io::Cursor;

/// Reads a number of bytes to some type.
///
/// A common use would be to read n bytes to an n-length slice, ex:
///
/// ```
/// let guid: [u8; 16] = read_bytes(rdr, 16);
/// ```
///
/// Note that the length of the slice and the nubmer of bytes read should be
/// the same, otherwise the function call can fail.
pub fn read_bytes<T: TryFrom<Vec<u8>>>(rdr: &mut Cursor<Vec<u8>>, len: usize) -> Result<T> {
  let mut buf = vec![0; len];
  rdr.read_exact(&mut buf)?;
  match T::try_from(buf) {
    Ok(t) => Ok(t),
    Err(_) => bail!("read_bytes({}) failed to convert to desired type", len),
  }
}

pub fn read_u32(rdr: &mut Cursor<Vec<u8>>) -> Result<u32> {
  Ok(rdr.read_u32::<LittleEndian>()?)
}

/// Reads a length-prefixed, null-terminated string
pub fn read_string(rdr: &mut Cursor<Vec<u8>>) -> Result<String> {
  let length = read_u32(rdr)? as usize;
  let chars = read_bytes(rdr, length - 1)?;
  rdr.consume(1); // Skip the 0 terminator
  Ok(String::from_utf8(chars)?)
}

pub fn read_bool(rdr: &mut Cursor<Vec<u8>>) -> Result<bool> {
  Ok(read_u32(rdr)? != 0)
}

pub fn write_u32(curs: &mut Cursor<Vec<u8>>, val: u32) -> Result<()> {
  curs.write_u32::<LittleEndian>(val)?;
  Ok(())
}

pub fn write_string(curs: &mut Cursor<Vec<u8>>, string: &str) -> Result<()> {
  let length = string.len() + 1;
  write_u32(curs, length as u32)?;
  curs.write(string.as_bytes())?;
  curs.write(&[0])?;
  Ok(())
}

pub fn write_bool(curs: &mut Cursor<Vec<u8>>, val: bool) -> Result<()> {
  write_u32(curs, if val { 1 } else { 0 })?;
  Ok(())
}
