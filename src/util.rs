use byteorder::{LittleEndian, ReadBytesExt};
use std::io::prelude::*;
use std::io::Cursor;

pub fn read_bytes(rdr: &mut Cursor<Vec<u8>>, len: usize) -> Vec<u8> {
  let buf = &mut (vec![0; len]);
  rdr.read_exact(buf).unwrap();
  return buf.to_vec();
}

pub fn read_u32(rdr: &mut Cursor<Vec<u8>>) -> u32 {
  rdr.read_u32::<LittleEndian>().unwrap()
}

pub fn read_string(rdr: &mut Cursor<Vec<u8>>) -> String {
  let length = read_u32(rdr) as usize;
  let chars = read_bytes(rdr, length - 1);
  rdr.consume(1); // Skip the 0 terminator
  return String::from_utf8(chars).unwrap();
}

pub fn read_bool(rdr: &mut Cursor<Vec<u8>>) -> bool {
  read_u32(rdr) != 0
}

pub fn peek_bytes(rdr: &mut Cursor<Vec<u8>>, len: usize) -> Vec<u8> {
  let mut buf = vec!();
  for _ in 0..len {
    let byte = *rdr.get_mut().iter().next().unwrap();
    buf.push(byte)
  }
  buf.to_vec()
}
