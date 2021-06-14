// #![windows_subsystem = "windows"]

mod asset;
mod bindings;
mod editor;
mod reader;
mod util;

use asset::*;
use editor::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::*;
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "help" {
        println!("Usage: drg-editor.exe [test|all] [filename]");
        println!("");
        println!("If the test/all positional argument is given, filename must also be passed.");
        println!("");
        println!("Options:");
        println!("  -r - For test mod, recursively searches the directory and tests all files");
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
    } else if args.len() == 3 && args[1] == "all" {
        // All mode

        let mut stream = BufWriter::new(File::create("out/all.txt").unwrap());

        let mut steps = 0;
        let mut total = 0;
        let mut failed = 0;
        for entry in WalkDir::new(args[2].clone()) {
            if steps % 1000 == 0 {
                println!(
                    "Processed {} entries ({}/{} are assets)",
                    steps, total, steps
                );
            }
            steps += 1;

            let entry = entry.unwrap();
            let fp = entry.path();
            if fp.extension() == Some("uasset".as_ref()) {
                total += 1;
                match &mut Asset::read_from(fp) {
                    Err(err) => {
                        failed += 1;
                        write!(&mut stream, "FAILED {}: {:?}\n", fp.display(), err).unwrap();
                    }
                    Ok(_) => {
                        write!(&mut stream, "SUCCESS {}\n", fp.display()).unwrap();
                    }
                }
            }
        }

        write!(
            &mut stream,
            "\n{}% failed ({}/{})\n",
            (failed as f32) / (total as f32) * 100.0,
            failed,
            total
        )
        .unwrap();

        return;
    }

    if args.len() == 2 {
        let asset_loc: &Path = args[1].as_ref();
        start_editor_with_path(asset_loc);
    } else {
        start_editor_empty();
    }
}
