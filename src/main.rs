mod sifis;
use crate::sifis::{get_sifis, SifisError};
use std::path::PathBuf;
fn main() -> Result<(), SifisError> {
    let mut pathf = PathBuf::new();
    let mut pathj = PathBuf::new();
    pathf.push("../rust-data-structures-main/src");
    pathj.push("./data/coveralls.json");
    match get_sifis(&pathf, &pathj, "../rust-data-structures-main/") {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}
