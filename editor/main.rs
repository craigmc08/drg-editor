#![windows_subsystem = "windows"]

#[macro_use]
extern crate clap;

mod editor;
mod internal;
mod keyboard;
mod operations;
mod plugins;
mod property_creator;
mod property_editor;
mod support;
mod tools;

use drg::asset::*;
use editor::*;
use std::path::*;

fn main() {
  let matches = clap_app!(DRGEditor =>
    (version: "0.1.0")
    (author: "Craig M. <craigmc08@gmail.com>")
    (@arg DATA: -d --data +takes_value "Directory for config files; default: ./data")
    (@arg ASSET: +takes_value "Asset to open; if not present, no asset is opened")
  )
  .get_matches();

  let data_dir: &Path = matches.value_of("DATA").unwrap_or("./data").as_ref();
  let struct_pattern_file = data_dir.join("struct-patterns.json");
  if let Err(err) = struct_pattern::StructPatterns::load(&struct_pattern_file) {
    println!("Failed to load struct patterns: {:?}", err);
    std::process::exit(-1);
  }

  if let Some(asset_loc) = matches.value_of("ASSET") {
    start_editor_with_path(asset_loc.as_ref());
  } else {
    start_editor_empty();
  }
}
