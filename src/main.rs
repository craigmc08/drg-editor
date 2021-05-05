// #![windows_subsystem = "windows"]

mod asset;
// mod bindings;
// mod editor;
mod util;

use asset::*;
// use editor::*;
use std::env;
use std::path::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "help" {
        println!("Usage: drg-editor.exe [test] [filename]");
        println!("");
        println!("If the test positional argument is given, filename must also be passed.");
        println!("In test mode, the program will read from the file and write to out/out.uasset");
        println!("");
        println!("Options:");
        println!("  help - Print this message");
        return;
    }

    if args.len() == 3 && args[1] == "test" {
        // Testing mode

        match &mut Asset::read_from(args[2].as_ref()) {
            Err(err) => {
                println!("Failed to read asset");
                println!("{:?}", err);
            }
            Ok(asset) => {
                asset.recalculate_offsets();
                match asset.write_out("out/out.uasset".as_ref()) {
                    Err(err) => {
                        println!("Failed to write asset");
                        println!("{:?}", err);
                    }
                    Ok(_) => {}
                }
            }
        }

        return;
    }

    let asset = if args.len() == 2 {
        let asset_loc: &Path = args[1].as_ref();
        let asset = Asset::read_from(asset_loc);
        Some(asset)
    } else {
        None
    };
    // start_editor(asset.map(|x| x.unwrap()));
}
