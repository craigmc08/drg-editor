use crate::file_summary::*;
use crate::object_imports::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct PreloadDependencies {
  pub dependencies: Vec<String>, // for each value (-n - 1) in dependencies, n is an index into object_imports
}

impl PreloadDependencies {
  pub fn read(
    rdr: &mut Cursor<Vec<u8>>,
    summary: &FileSummary,
    imports: &ObjectImports,
  ) -> Result<Self, String> {
    if rdr.position() != summary.preload_dependency_offset.into() {
      return Err(
        format!(
          "Error parsing PreloadDependencies: Expected to be at position {}, but I'm at position {}",
          summary.preload_dependency_offset,
          rdr.position()
        )
        .to_string(),
      );
    }

    let mut dependencies = vec![];
    for _ in 0..summary.preload_dependency_count {
      let import =
        imports.read_import(rdr, &format!("preload dependency @ {:04X}", rdr.position()))?;
      dependencies.push(import);
    }

    return Ok(PreloadDependencies { dependencies });
  }

  pub fn byte_size(&self) -> usize {
    // 4 bytes per string index
    self.dependencies.len() * 4
  }
}
