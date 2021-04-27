pub mod util;
pub mod asset;
pub mod editor;

use asset::*;
use std::env;
use std::path::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Expected 1 argument");
        return;
    }

    let asset_loc: &Path = args[1].as_ref();
    let asset_name: &Path = asset_loc.file_name().unwrap().as_ref();
    let uassetfp = asset_loc.with_extension("uasset");
    let uexpfp = asset_loc.with_extension("uexp");
    println!("Reading from {} and {}", uassetfp.display(), uexpfp.display());

    match std::fs::read(uassetfp.clone()).and_then(|uasset| {
        let uexp = std::fs::read(uexpfp.clone())?;
        Ok((uasset, uexp))
    }) {
        Err(e) => {
            println!("{}", e);
        }
        Ok((uasset, uexp)) => {
            let mut asset = Asset::read(uasset, uexp);

            println!("{:#?}", asset.summary);
            println!("{:#?}", asset.names);
            println!("{:#?}", asset.imports);
            println!("{:#?}", asset.exports);
            println!("{:#?}", asset.assets);
            println!("{:#?}", asset.dependencies);
            println!("{:#?}", asset.structs);

            asset.import("/Script/CoreUObject", "Package", "/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun", Dependency::UObject);
            asset.import("/Script/FSD", "ItemID", "ID_GrapplingGun", Dependency::import("/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun"));

            let data = &mut asset.structs[0];
            let new_tools = vec![Dependency::import("ID_GrapplingGun")];
            data.set("TraversalTools", new_tools);

            // Make sure to recalculate offsets
            asset.recalculate_offsets();

            let (uasset_out, uexp_out) = asset.write();

            let mut asset_out_loc = PathBuf::from("out\\");
            asset_out_loc.push(asset_name);
            let uasset_outfp = asset_out_loc.with_extension("uasset");
            let uexp_outfp = asset_out_loc.with_extension("uexp");

            println!("Writing to {} and {}", uasset_outfp.display(), uexp_outfp.display());

            std::fs::write(uasset_outfp, uasset_out).unwrap();
            std::fs::write(uexp_outfp, uexp_out).unwrap();
        }
    }
}
