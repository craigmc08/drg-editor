pub mod util;
pub mod asset;

use asset::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Expected 1 argument");
        return;
    }

    let uassetfp = format!("{}.uasset", args[1]);
    let uexpfp = format!("{}.uexp", args[1]);
    println!("Reading from {} and {}", uassetfp, uexpfp);

    match std::fs::read(uassetfp.clone()).and_then(|uasset| {
        let uexp = std::fs::read(uexpfp.clone())?;
        Ok((uasset, uexp))
    }) {
        Err(e) => {
            println!("{}", e);
        }
        Ok((uasset, uexp)) => {
            let asset = Asset::read(uasset, uexp).unwrap();
            println!("{:#?}", asset.summary);
            println!("{:#?}", asset.names);
            println!("{:#?}", asset.imports);
            println!("{:#?}", asset.exports);
            println!("{:#?}", asset.assets);
            println!("{:#?}", asset.dependencies);
            println!("{:#?}", asset.properties);

            let (uasset_out, uexp_out) = asset.write();

            std::fs::write(format!("{}.out", uassetfp), uasset_out).unwrap();
            std::fs::write(format!("{}.out", uexpfp), uexp_out).unwrap();
        }
    }
}
