use rust_code_analysis::{metrics, read_file, ParserTrait, RustParser};
use serde_json::*;
use std::collections::*;
use std::fs;
use std::path::PathBuf;

fn sifis_plain(path: &PathBuf, covs: Vec<Value>) -> f64 {
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
    sifis
}
fn read_json(file: String, prefix: &str) -> HashMap<String, Vec<Value>> {
    let val: Value = serde_json::from_str(file.as_str()).unwrap();
    let vec = val["source_files"].as_array().unwrap();
    let mut covs = HashMap::<String, Vec<Value>>::new();
    for x in vec {
        let mut name = prefix.to_string();
        name = name + x["name"].as_str().unwrap();
        covs.insert(name.to_string(), x["coverage"].as_array().unwrap().to_vec());
    }
    covs
}
fn main() {
    let paths = fs::read_dir("../rust-data-structures-main/src").unwrap();
    let file = fs::read_to_string("./src/coveralls.json").unwrap();
    let covs = read_json(file,"../rust-data-structures-main/");
    for path in paths {
        let p = path.unwrap().path();
        let key =p.to_str().unwrap().to_string();
        let arr = covs.get(&key).unwrap().to_vec();
        let sifis =sifis_plain(&p, arr);
        println!(
            "For {:?} the SIFIS plain value is {:?}",
            p.file_name().unwrap(),
            sifis
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_read_json(){
    let file = fs::read_to_string("./test/test.json").unwrap();
    let covs = read_json(file,"../rust-data-structures-main/");
    assert_eq!(covs.contains_key("../rust-data-structures-main/test/test.rs"), true);
    assert_eq!(covs.contains_key("../rust-data-structures-main/src/main.rs"), true);
    let vec= covs.get("../rust-data-structures-main/test/test.rs").unwrap();
    assert_eq!(vec.len(),12);
    let vec_main= covs.get("../rust-data-structures-main/src/main.rs").unwrap();
    assert_eq!(vec_main.len(),9);
    let  value = vec.get(6).unwrap();
    assert_eq!(value,2);
    let  value_null = vec.get(1).unwrap();
    assert_eq!(value_null.is_null(),true);
    }
    
    #[test]
    fn test_sifis_plain(){
    let file = fs::read_to_string("./test/test.json").unwrap();
    let covs = read_json(file,"../rust-data-structures-main/");   
    let mut path = PathBuf::new();
    path.push("./test/test.rs");
    let sifis = sifis_plain(&path,covs.get("../rust-data-structures-main/test/test.rs").unwrap().to_vec());
    assert_eq!(sifis,24./10.)
    }
}
