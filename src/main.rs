// #![windows_subsystem = "windows"]

#[macro_use]
extern crate clap;

mod asset;
mod bindings;
mod editor;
mod reader;
mod util;

use anyhow::*;
use asset::*;
use editor::*;
use rayon::iter::*;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::*;
use walkdir::WalkDir;

fn main() {
  let matches = clap_app!(DRGEditor =>
    (version: "0.1.0")
    (author: "Craig M. <craigmc08@gmail.com>")
    (@arg DATA: -d --data +takes_value "Directory for config files; default: ./data")
    (@subcommand gui =>
      (about: "Open graphical editor")
      (@arg ASSET: +takes_value "Asset to open; if not present, no asset is opened")
    )
    (@subcommand test =>
      (about: "Deserializes and serializes a single asset file")
      (@arg OUT: -o --out +takes_value "Filename to serialize asset to; default: ./out/out.[uasset/uexp]")
      (@arg ASSET: +takes_value +required "Path to asset to test")
    )
    (@subcommand all =>
      (about: "Deserializes every asset file recursively in a directory")
      (@arg OUT: -o --out +takes_value "Filename to output test information about; if not present, prints to stdout")
      (@arg DIRECTORY: +takes_value +required "Path to directory containing assets")
    )
  ).get_matches();

  let data_dir: &Path = matches.value_of("DATA").unwrap_or("./data").as_ref();
  let struct_pattern_file = data_dir.join("struct-patterns.json");
  if let Err(err) = struct_pattern::StructPatterns::load(&struct_pattern_file) {
    println!("Failed to load struct patterns: {:?}", err);
    std::process::exit(-1);
  }

  if let Some(matches) = matches.subcommand_matches("gui") {
    if let Some(asset_loc) = matches.value_of("ASSET") {
      start_editor_with_path(asset_loc.as_ref());
    } else {
      start_editor_empty();
    }
  } else if let Some(matches) = matches.subcommand_matches("test") {
    let out_file = matches.value_of("OUT").unwrap_or("./out/out");
    let asset_loc = matches.value_of("ASSET").unwrap();
    test_command(out_file, asset_loc);
    return;
  } else if let Some(matches) = matches.subcommand_matches("all") {
    let out_file = matches.value_of("OUT");
    let dir = matches.value_of("DIRECTORY").unwrap();
    all_command(out_file, dir);
    return;
  }
}

fn test_command(out_file: &str, asset_loc: &str) {
  if let Err(err) = Asset::test_rw(asset_loc.as_ref()) {
    println!("Error testing r/w of asset");
    println!("{:?}", err);
  }

  match &mut Asset::read_from(asset_loc.as_ref()) {
    Err(err) => {
      println!("Failed to read asset");
      println!("{:?}", err);
      std::process::exit(-1);
    }
    Ok(asset) => {
      asset.recalculate_offsets();
      if let Err(err) = asset.write_out(out_file.as_ref()) {
        println!("Failed to write asset");
        println!("{:?}", err);
      }
    }
  }
}

fn all_command(out_file: Option<&str>, dir: &str) {
  let show_info = out_file.is_some();

  let asset_locs: Vec<PathBuf> = WalkDir::new(dir.clone())
    .into_iter()
    .map(|entry| entry.unwrap().into_path())
    .filter(|fp| fp.extension() == Some("uasset".as_ref()))
    .collect();

  let total = asset_locs.len();
  if show_info {
    println!("Number of assets: {}", total);
  }

  let mut results: Vec<(PathBuf, Result<()>)> = vec![];
  asset_locs
    .into_par_iter()
    .enumerate()
    .map(|(i, fp)| {
      if i % 1000 == 0 && show_info {
        println!("Processed {}/{} assets", i, total);
      }
      (fp.clone(), Asset::test_rw(fp.as_ref()))
    })
    .collect_into_vec(&mut results);

  let mut out_stream = if let Some(out_file) = out_file {
    BufWriter::new(Box::new(std::fs::File::create(out_file).unwrap()) as Box<dyn Write>)
  } else {
    BufWriter::new(Box::new(std::io::stdout()) as Box<dyn Write>)
  };

  results.iter().for_each(|(fp, result)| match result {
    Err(err) => {
      writeln!(
        &mut out_stream,
        "ASSET {}\nFAILURE\n{:?}\n====================",
        fp.display(),
        err
      )
      .unwrap();
    }
    Ok(_) => {
      writeln!(
        &mut out_stream,
        "ASSET {}\nSUCCESS\n====================",
        fp.display()
      )
      .unwrap();
    }
  });

  let success_count = results
    .iter()
    .filter(|(entry, result)| result.is_ok())
    .count();
  writeln!(
    &mut out_stream,
    "TOTAL\nSUCCESS {} of {}\nPERCENT {}%",
    success_count,
    total,
    (success_count as f32) / (total as f32)
  )
  .unwrap();
}
