use rust_code_analysis::{dump_root, metrics, read_file, ParserTrait, RustParser};
use serde_json::*;
use std::collections::*;
use std::fs;
use std::path::PathBuf;

fn sifis_plain(path: PathBuf, covs: Vec<Value>) {
    let data = read_file(&path).unwrap();
    let parser = RustParser::new(data, &path, None);
    let space = metrics(&parser, &path).unwrap();
    let ploc = space.metrics.loc.ploc();
    let comp = space.metrics.cyclomatic.cyclomatic_sum();
    let mut sum = 0.0;
    for i in 0..covs.len() {
        let is_null = covs.get(i).unwrap().is_null();

        if !is_null {
            let cov = covs.get(i).unwrap().as_u64().unwrap();
            if cov > 0 {
                sum += comp;
            }
        }
    }
    let sifis = sum / ploc;
    println!(
        "For {:?} the SIFIS plain value is {:?}",
        path.file_name().unwrap(),
        sifis
    );
}
fn read_json(file: String) -> HashMap<String, Vec<Value>> {
    let val: Value = serde_json::from_str(file.as_str()).unwrap();
    let vec = val["source_files"].as_array().unwrap();
    let mut covs = HashMap::<String, Vec<Value>>::new();
    for x in vec {
        let mut name = "../rust-data-structures-main/".to_string();
        name = name + x["name"].as_str().unwrap();
        covs.insert(name.to_string(), x["coverage"].as_array().unwrap().to_vec());
    }
    covs
}
fn main() {
    let paths = fs::read_dir("../rust-data-structures-main/src").unwrap();
    let file = fs::read_to_string("./src/coveralls.json").unwrap();
    let covs = read_json(file);
    for path in paths {
        let p = path.unwrap().path();
        let key =p.to_str().unwrap().to_string();
        let arr = covs.get(&key).unwrap().to_vec();
        sifis_plain(p, arr)
    }
}
