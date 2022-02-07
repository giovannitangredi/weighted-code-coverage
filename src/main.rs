use sifis_test::sifis::get_sifis;
use std::path::PathBuf;
fn main() {
    let mut pathf = PathBuf::new();
    let mut pathj = PathBuf::new();
    pathf.push("../rust-data-structures-main/src");
    pathj.push("./data/coveralls.json");
    match get_sifis(&pathf, &pathj, "../rust-data-structures-main/") {
        Ok(()) => (),
        Err(err) => println!("{}", err),
    }
}
