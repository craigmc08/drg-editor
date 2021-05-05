use crate::asset::*;
use std::io::Cursor;

pub type Curs<'a> = &'a mut Cursor<Vec<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct PropertyContext<'a> {
  pub summary: &'a FileSummary,
  pub names: &'a NameMap,
  pub imports: &'a ObjectImports,
  pub exports: &'a ObjectExports,
}

impl<'a> PropertyContext<'a> {
  pub fn new(
    summary: &'a FileSummary,
    names: &'a NameMap,
    imports: &'a ObjectImports,
    exports: &'a ObjectExports,
  ) -> Self {
    Self {
      summary,
      names,
      imports,
      exports,
    }
  }
}
