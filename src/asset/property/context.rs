use crate::asset::*;
use crate::struct_pattern::*;

#[derive(Debug, Clone, Copy)]
pub struct PropertyContext<'a> {
  pub summary: &'a FileSummary,
  pub names: &'a Names,
  pub imports: &'a Imports,
  pub exports: &'a Exports,
  pub patterns: &'a StructPatterns,
}

impl<'a> PropertyContext<'a> {
  pub fn new(
    summary: &'a FileSummary,
    names: &'a Names,
    imports: &'a Imports,
    exports: &'a Exports,
    patterns: &'a StructPatterns,
  ) -> Self {
    Self {
      summary,
      names,
      imports,
      exports,
      patterns,
    }
  }
}
