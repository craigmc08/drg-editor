use crate::asset::*;
use crate::reader::*;
use anyhow::*;
use std::io::Cursor;

#[derive(Debug)]
pub struct PreloadDependencies {
  // each n in the dependencies array is:
  // - negative? then -n - 1 is the index into imports
  // - positive? then n - 1 is the index into exports
  pub dependencies: Vec<Reference>,
}

impl PreloadDependencies {
  pub fn read(
    rdr: &mut ByteReader,
    summary: &FileSummary,
    imports: &Imports,
    exports: &Exports,
  ) -> Result<Self> {
    if rdr.position() != summary.preload_dependency_offset as u64 {
      bail!(
        "Wrong preload dependencies position: Expected to be at position {:#X}, but I'm at position {:#X}",
        summary.preload_dependency_offset,
        rdr.position(),
      );
    }

    let mut dependencies = vec![];
    for _ in 0..summary.preload_dependency_count {
      let start_pos = rdr.position();
      let reference = Reference::read(rdr, imports, exports)
        .with_context(|| format!("Failed to parse Reference starting at {:#X}", start_pos))?;
      dependencies.push(reference);
    }

    Ok(PreloadDependencies { dependencies })
  }

  pub fn write(
    &self,
    curs: &mut Cursor<Vec<u8>>,
    names: &Names,
    imports: &Imports,
    exports: &Exports,
  ) -> Result<()> {
    for dep in self.dependencies.iter() {
      dep.write(curs, names, imports, exports)?;
    }
    Ok(())
  }

  pub fn byte_size(&self) -> usize {
    // 4 bytes per string index
    self.dependencies.len() * 4
  }
}
