#![windows_subsystem = "windows"]

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
use std::path::*;

fn main() {
  let matches = clap_app!(DRGEditor =>
    (version: "0.1.0")
    (author: "Craig M. <craigmc08@gmail.com>")
    (@arg DATA: -d --data +takes_value "Directory for config files; default: ./data")
    (@arg ASSET: +takes_value "Asset to open; if not present, no asset is opened")
  )
  .get_matches();

  let data_dir = matches.value_of("DATA").unwrap_or("./data");
  if let Some(asset_loc) = matches.value_of("ASSET") {
    start_editor_with_path(asset_loc.as_ref());
  } else {
    start_editor_empty();
  }
}
