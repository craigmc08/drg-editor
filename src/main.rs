pub mod asset;
pub mod bindings;
pub mod editor;
pub mod util;

use asset::*;
use editor::*;
use std::env;
use std::path::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let asset = if args.len() == 2 {
        let asset_loc: &Path = args[1].as_ref();
        let asset = Asset::read_from(asset_loc);
        Some(asset)
    } else {
        None
    };
    start_editor(asset);
}
