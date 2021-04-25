pub mod asset_registry;
pub mod export_map;
pub mod file_summary;
pub mod name_map;
pub mod object_imports;
pub mod preload_dependencies;
pub mod util;
pub mod data;
pub mod property;

use asset_registry::*;
use export_map::*;
use file_summary::*;
use name_map::*;
use object_imports::*;
use preload_dependencies::*;
use property::*;
use std::env;
use std::io::Cursor;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Expected 1 argument");
        return;
    }

    let uassetfp = format!("{}.uasset", args[1]);
    let uexpfp = format!("{}.uexp", args[1]);
    println!("Reading from {} and {}", uassetfp, uexpfp);

    match std::fs::read(uassetfp).and_then(|uasset| {
        let uexp = std::fs::read(uexpfp)?;
        Ok((uasset, uexp))
    }) {
        Err(e) => {
            println!("{}", e);
        }
        Ok((uasset, uexp)) => {
            let mut rdr = Cursor::new(uasset);
            let summary = FileSummary::read(&mut rdr);
            let names = NameMap::read(&mut rdr, &summary).unwrap();
            let imports = ObjectImports::read(&mut rdr, &summary, &names).unwrap();
            let exports = ObjectExports::read(&mut rdr, &summary, &names, &imports).unwrap();
            let assets = AssetRegistry::read(&mut rdr, &summary).unwrap();
            let dependencies = PreloadDependencies::read(&mut rdr, &summary, &imports).unwrap();
            println!("{:#?}", summary);
            println!("{:#?}", names);
            println!("{:#?}", imports);
            println!("{:#?}", exports);
            println!("{:#?}", assets);
            println!("{:#?}", dependencies);

            assert_eq!(summary.byte_size(), summary.name_offset as usize);
            assert_eq!(
                summary.byte_size() + names.byte_size(),
                summary.import_offset as usize
            );
            assert_eq!(
                summary.byte_size() + names.byte_size() + imports.byte_size(),
                summary.export_offset as usize
            );
            assert_eq!(
                summary.byte_size()
                    + names.byte_size()
                    + imports.byte_size()
                    + exports.byte_size()
                    + 4,
                summary.asset_registry_data_offset as usize
            );
            assert_eq!(
                summary.byte_size()
                    + names.byte_size()
                    + imports.byte_size()
                    + exports.byte_size()
                    + assets.byte_size()
                    + dependencies.byte_size(),
                summary.total_header_size as usize
            );

            let mut rdruexp = Cursor::new(uexp);
            let properties = Property::read_uexp(&mut rdruexp, &names, &imports).unwrap();
            println!("{:#?}", properties);

            assert_eq!(Property::struct_size(&properties), 591);

            assert_eq!(
                summary.byte_size()
                    + names.byte_size()
                    + imports.byte_size()
                    + exports.byte_size()
                    + assets.byte_size()
                    + dependencies.byte_size()
                    + Property::struct_size(&properties),
                summary.bulk_data_start_offset as usize,
            );
        }
    }
}
