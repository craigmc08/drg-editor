use std::io::Read;
use std::io::{Seek, SeekFrom};

pub struct ByteReader {
  vec: Vec<u8>,
  current: usize,
  end: Option<usize>,
  end_stack: Vec<usize>,
}

impl ByteReader {
  pub fn new(vec: Vec<u8>) -> Self {
    Self {
      vec,
      current: 0,
      end: None,
      end_stack: vec![],
    }
  }

  pub fn limit(&mut self, size: usize) {
    if let Some(end) = self.end {
      self.end_stack.push(end);
    }
    self.end = Some(self.current + size);
  }

  pub fn unlimit(&mut self) {
    self.end = self.end_stack.pop();
  }

  pub fn position(&self) -> u64 {
    self.current as u64
  }

  pub fn at_end(&self) -> bool {
    match self.end {
      None => false,
      Some(end) => self.current >= end,
    }
  }

  pub fn remaining_bytes(&self) -> usize {
    match self.end {
      None => self.vec.len() - self.current,
      Some(end) => end - self.current,
    }
  }
}

impl Read for ByteReader {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    let in_limit = self.end.unwrap_or(self.vec.len());
    let out_limit = buf.len();

    let mut i = 0;
    while i < out_limit && self.current < in_limit {
      buf[i] = self.vec[self.current];

      i += 1;
      self.current += 1;
    }

    Ok(i)
  }
}

impl Seek for ByteReader {
  fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
    match pos {
      SeekFrom::Start(p) => {
        self.current = p as usize;
      }
      SeekFrom::Current(p) => {
        self.current = (self.current as i64 + p) as usize;
      }
      SeekFrom::End(p) => {
        self.current = (self.vec.len() as i64 + p) as usize;
      }
    }
    Ok(self.current as u64)
  }
}
