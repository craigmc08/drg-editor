// #![windows_subsystem = "windows"]

#[macro_use]
extern crate clap;
use clap::App;

mod asset;
mod bindings;
mod editor;
mod reader;
mod util;

use asset::*;
use editor::*;
use std::fs::File;
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

  let data_dir = matches.value_of("DATA").unwrap_or("./data");

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
  match &mut Asset::read_from(asset_loc.as_ref()) {
    Err(err) => {
      println!("Failed to read asset");
      println!("{:?}", err);
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
  let mut out_stream = if let Some(out_file) = out_file {
    BufWriter::new(Box::new(std::fs::File::create(out_file).unwrap()) as Box<dyn Write>)
  } else {
    BufWriter::new(Box::new(std::io::stdout()) as Box<dyn Write>)
  };

  let mut total = 0;
  let mut failed = 0;
  for (steps, entry) in WalkDir::new(dir.clone()).into_iter().enumerate() {
    if steps % 1000 == 0 {
      println!(
        "Processed {} entries ({}/{} are assets)",
        steps, total, steps
      );
    }

    let entry = entry.unwrap();
    let fp = entry.path();
    if fp.extension() == Some("uasset".as_ref()) {
      total += 1;
      match &mut Asset::read_from(fp) {
        Err(err) => {
          failed += 1;
          writeln!(&mut out_stream, "FAILED {}: {:?}", fp.display(), err).unwrap();
        }
        Ok(_) => {
          writeln!(&mut out_stream, "SUCCESS {}", fp.display()).unwrap();
        }
      }
    }
  }

  writeln!(
    &mut out_stream,
    "\n{}% failed ({}/{})",
    (failed as f32) / (total as f32) * 100.0,
    failed,
    total
  )
  .unwrap();
}
