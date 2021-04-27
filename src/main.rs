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
    let mut asset = Asset::read_from(asset_loc);

    asset.import("/Script/CoreUObject", "Package", "/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun", Dependency::UObject);
    asset.import("/Script/FSD", "ItemID", "ID_GrapplingGun", Dependency::import("/Game/WeaponsNTools/GrapplingGun/ID_GrapplingGun"));

    let data = &mut asset.structs[0];
    let new_tools = vec![Dependency::import("ID_GrapplingGun")];
    data.set("TraversalTools", new_tools);

    // Make sure to recalculate offsets before writing
    asset.recalculate_offsets();
    asset.write_out(asset_loc);
}
