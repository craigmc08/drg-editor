use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
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

pub fn write_u32(curs: &mut Cursor<Vec<u8>>, val: u32) -> () {
  curs.write_u32::<LittleEndian>(val).unwrap();
}

pub fn write_string(curs: &mut Cursor<Vec<u8>>, string: &str) -> () {
  let length = string.len() + 1;
  write_u32(curs, length as u32);
  curs.write(string.as_bytes()).unwrap();
  curs.write(&[0]).unwrap();
}

pub fn write_bool(curs: &mut Cursor<Vec<u8>>, val: bool) -> () {
  write_u32(curs, if val { 1 } else { 0 });
}
