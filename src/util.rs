use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::io::{Seek, SeekFrom};

/// Reads a number of bytes to some type.
///
/// A common use would be to read n bytes to an n-length slice, ex:
///
/// ```ignore
/// let guid: [u8; 16] = read_bytes(rdr, 16);
/// ```
///
/// Note that the length of the slice and the nubmer of bytes read should be
/// the same, otherwise the function call can fail.
pub fn read_bytes<R: Read, T: TryFrom<Vec<u8>>>(rdr: &mut R, len: usize) -> Result<T> {
  let mut buf = vec![0; len];
  rdr.read_exact(&mut buf)?;
  match T::try_from(buf) {
    Ok(t) => Ok(t),
    Err(_) => bail!("read_bytes({}) failed to convert to desired type", len),
  }
}

pub fn read_u32<R: Read>(rdr: &mut R) -> Result<u32> {
  Ok(rdr.read_u32::<LittleEndian>()?)
}

pub fn read_byte_string<R: Read>(rdr: &mut R) -> Result<Vec<u8>> {
  let length = read_u32(rdr)? as usize;
  if length == 0 {
    bail!("Cannot read byte string with length 0 at (figure out how to?)")
  } else {
    let chars = read_bytes(rdr, length - 1)?;
    rdr.read_exact(&mut [0])?; // Skip past 0 terminator
    Ok(chars)
  }
}

/// Reads a length-prefixed, null-terminated string
pub fn read_string<R: Read>(rdr: &mut R) -> Result<String> {
  read_byte_string(rdr).and_then(|bytes| Ok(String::from_utf8(bytes)?))
}

pub fn read_bool<R: Read>(rdr: &mut R) -> Result<bool> {
  Ok(read_u32(rdr)? != 0)
}

pub fn next_matches<R: Read + Seek>(rdr: &mut R, bytes: &[u8]) -> bool {
  // This unwrap should never fail
  let start_pos = rdr.seek(SeekFrom::Current(0)).unwrap();
  if let Ok(read) = read_bytes::<R, Vec<u8>>(rdr, bytes.len()) {
    let eq = bytes.iter().zip(read).all(|(a, b)| *a == b);
    rdr
      .seek(SeekFrom::Start(start_pos))
      .expect("Seek should not fail");
    eq
  } else {
    false
  }
}

pub fn write_u32<W: Write>(curs: &mut W, val: u32) -> Result<()> {
  curs.write_u32::<LittleEndian>(val)?;
  Ok(())
}

pub fn write_byte_string<W: Write, B: AsRef<[u8]>>(curs: &mut W, bytes: B) -> Result<()> {
  let bytes = bytes.as_ref();
  let length = bytes.len() + 1;
  write_u32(curs, length as u32)?;
  curs.write_all(bytes)?;
  curs.write_all(&[0])?;
  Ok(())
}

pub fn write_string<W: Write>(curs: &mut W, string: &str) -> Result<()> {
  write_byte_string(curs, string.as_bytes())
}

pub fn write_bool<W: Write>(curs: &mut W, val: bool) -> Result<()> {
  write_u32(curs, if val { 1 } else { 0 })?;
  Ok(())
}
