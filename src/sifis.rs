use rust_code_analysis::{metrics, read_file, FuncSpace, ParserTrait, RustParser};
use serde_json::{ Value};
use std::collections::*;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SifisError {
    #[error("Error while reading File: {0}")]
    WrongFile(String),
    #[error("Error while converting JSON value to a type")]
    ConversionError(),
    #[error("Error while taking value from HashMap with key : {0}")]
    HashMapError(String),
    #[error("Failing reading JSON from string")]
    ReadingJSONError(),
    #[error("Error while computing Metrics")]
    MetricsError(),
}
fn read_json(file: String, prefix: &str) -> Result<HashMap<String, Vec<Value>>, SifisError> {
    let val: Value = match serde_json::from_str(file.as_str()) {
        Ok(val) => val,
        Err(_err) => return Err(SifisError::ReadingJSONError()),
    };
    let vec = match val["source_files"].as_array() {
        Some(vec) => vec,
        None => return Err(SifisError::ReadingJSONError()),
    };
    let mut covs = HashMap::<String, Vec<Value>>::new();
    for x in vec {
        let mut name = prefix.to_string();
        name += x["name"].as_str().unwrap();
        let value = match x["coverage"].as_array() {
            Some(value) => value.to_vec(),
            None => return Err(SifisError::ConversionError()),
        };
        covs.insert(name.to_string(), value);
    }
    Ok(covs)
}

fn get_min_space(root: &FuncSpace, i: usize) -> FuncSpace {
    let mut min_space: FuncSpace = root.clone();
    let mut stack: Vec<FuncSpace> = Vec::<FuncSpace>::new();
    stack.push(root.clone());
    while let Some(space) = stack.pop() {
        for s in space.spaces.into_iter() {
            if i >= s.start_line && i <= s.end_line {
                min_space = s.clone();
                stack.push(s);
            }
        }
    }
    min_space
}

fn sifis_plain(path: &PathBuf, covs: &Vec<Value>) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.as_path().display().to_string())),
    };
    let parser = RustParser::new(data, path, None);
    let space = match metrics(&parser, path) {
        Some(space) => space,
        None => return Err(SifisError::MetricsError()),
    };
    let ploc = space.metrics.loc.ploc();
    let comp = space.metrics.cyclomatic.cyclomatic_sum();
    let mut sum = 0.0;

    for i in 0..covs.len() {
        let is_null = match covs.get(i) {
            Some(val) => val.is_null(),
            None => return Err(SifisError::ConversionError()),
        };

        if !is_null {
            let cov = match covs.get(i).unwrap().as_u64() {
                Some(cov) => cov,
                None => return Err(SifisError::ConversionError()),
            };
            if cov > 0 {
                sum += comp;
            }
        }
    }
    Ok(sum / ploc)
}

fn sifis_quantized(path: &PathBuf, covs: &Vec<Value>) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.as_path().display().to_string())),
    };
    let parser = RustParser::new(data, path, None);
    let root = match metrics(&parser, path) {
        Some(root) => root,
        None => return Err(SifisError::MetricsError()),
    };
    let ploc = root.metrics.loc.ploc();
    let mut sum = 0.0;
    let threshold = 10.;
    //for each line find the minimun space and get complexity value then sum 1 if comp>thresholdelse sum 1
    for i in 0..covs.len() {
        let is_null = match covs.get(i) {
            Some(val) => val.is_null(),
            None => return Err(SifisError::ConversionError()),
        };

        if !is_null {
            let cov = match covs.get(i).unwrap().as_u64() {
                Some(cov) => cov,
                None => return Err(SifisError::ConversionError()),
            };
            if cov > 0 {
                let min_space: FuncSpace = get_min_space(&root, i);
                let comp = min_space.metrics.cyclomatic.cyclomatic();
                if comp > threshold {
                    sum += 2.;
                } else {
                    sum += 1.;
                }
            }
        }
    }
    Ok(sum / ploc)
}

pub fn get_sifis(
    files_path: &PathBuf,
    json_path: &PathBuf,
    prefix: &str,
) -> Result<(), SifisError> {
    let paths = match fs::read_dir(files_path) {
        Ok(paths) => paths,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_path().display().to_string(),
            ))
        }
    };
    let file = match fs::read_to_string(json_path) {
        Ok(file) => file,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                json_path.as_path().display().to_string(),
            ))
        }
    };
    let covs = read_json(file, prefix)?;
    for path in paths {
        let p = path.unwrap().path();
        let key = p.to_str().unwrap().to_string();
        let arr = match covs.get(&key) {
            Some(arr) => arr.to_vec(),
            None => return Err(SifisError::HashMapError(key)),
        };
        let sifis = sifis_plain(&p, &arr)?;
        println!(
            "For {:?} the SIFIS plain value is {:.3?}",
            p.file_name().unwrap(),
            sifis
        );
        let sifis_quantized = sifis_quantized(&p, &arr)?;
        println!(
            "For {:?} the SIFIS quantized value is {:.3?}",
            p.file_name().unwrap(),
            sifis_quantized
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_read_json() {
        let file = fs::read_to_string("./data/data.json").unwrap();
        let covs = read_json(file, "../rust-data-structures-main/").unwrap();
        assert_eq!(
            covs.contains_key("../rust-data-structures-main/data/simple_main.rs"),
            true
        );
        assert_eq!(
            covs.contains_key("../rust-data-structures-main/data/main.rs"),
            true
        );
        let vec = covs
            .get("../rust-data-structures-main/data/simple_main.rs")
            .unwrap();
        assert_eq!(vec.len(), 12);
        let vec_main = covs
            .get("../rust-data-structures-main/data/main.rs")
            .unwrap();
        assert_eq!(vec_main.len(), 9);
        let value = vec.get(6).unwrap();
        assert_eq!(value, 2);
        let value_null = vec.get(1).unwrap();
        assert_eq!(value_null.is_null(), true);
    }

    #[test]
    fn test_sifis_plain() {
        let file = fs::read_to_string("./data/data.json").unwrap();
        let covs = read_json(file, "../rust-data-structures-main/").unwrap();
        let mut path = PathBuf::new();
        path.push("./data/simple_main.rs");
        let vec = covs
            .get("../rust-data-structures-main/data/simple_main.rs")
            .unwrap()
            .to_vec();
        let sifis = sifis_plain(&path, &vec).unwrap();
        assert_eq!(sifis, 24. / 10.)
    }

    #[test]
    fn test_sifis_quantized() {
        let file = fs::read_to_string("./data/data.json").unwrap();
        let covs = read_json(file, "../rust-data-structures-main/").unwrap();
        let mut path = PathBuf::new();
        path.push("./data/simple_main.rs");
        let vec = covs
            .get("../rust-data-structures-main/data/simple_main.rs")
            .unwrap()
            .to_vec();
        let sifis = sifis_quantized(&path, &vec).unwrap();
        assert_eq!(sifis, 6. / 10.)
    }
}
