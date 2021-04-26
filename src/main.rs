pub mod util;
pub mod asset;

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
            let mut asset = Asset::read(uasset, uexp).unwrap();

            // Add some data!
            asset.names.add("/Game/WeaponsNTools/Drills/ID_DoubleDrills");
            asset.names.add("ID_DoubleDrills");
            asset.names.add("/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun");
            asset.names.add("ID_GrapplingGun");
            asset.names.add("/Game/WeaponsNTools/PlatformGun/ID_PlatformGun");
            asset.names.add("ID_PlatformGun");

            let drill_idx = asset.imports.add("/Script/CoreUObject", "Package", "/Game/WeaponsNTools/Drills/ID_DoubleDrills", 0);
            let grapple_idx = asset.imports.add("/Script/CoreUObject", "Package", "/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun", 0);
            let platform_idx = asset.imports.add("/Script/CoreUObject", "Package", "/Game/WeaponsNTools/PlatformGun/ID_PlatformGun", 0);
            asset.imports.add("/Script/FSD", "ItemID", "ID_DoubleDrills", drill_idx);
            asset.imports.add("/Script/FSD", "ItemID", "ID_GrapplingGun", grapple_idx);
            asset.imports.add("/Script/FSD", "ItemID", "ID_PlatformGun", platform_idx);
            asset.dependencies.add("ID_DoubleDrills");
            asset.dependencies.add("ID_GrapplingGun");
            asset.dependencies.add("ID_PlatformGun");

            // Add Drills to TraversalTools
            let traversal_tools = Property::find(&mut asset.properties, "TraversalTools").expect("Expected TerrainTools property");
            match &mut traversal_tools.value {
                PropertyValue::ArrayProperty { values, .. } => {
                    values.push(PropertyValue::ObjectProperty { value: "ID_GrapplingGun".to_string() });
                    values.push(PropertyValue::ObjectProperty { value: "ID_DoubleDrills".to_string() });
                    values.push(PropertyValue::ObjectProperty { value: "ID_PlatformGun".to_string() });
                }
                _ => panic!("Expected TraversalTools to be an ArrayProperty")
            }

            asset.recalculate_offsets();
            // println!("{:#?}", asset.summary);
            // println!("{:#?}", asset.names);
            // println!("{:#?}", asset.imports);
            // println!("{:#?}", asset.exports);
            // println!("{:#?}", asset.assets);
            // println!("{:#?}", asset.dependencies);
            // println!("{:#?}", asset.properties);

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
