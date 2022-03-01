use rust_code_analysis::{metrics, read_file, FuncSpace, ParserTrait, RustParser};
use serde_json::Value;
use std::collections::*;
use std::fs;
use std::path::*;
use thiserror::Error;
/// Customized error messages using thiserror library
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

///This function read all  the files in the project folder
/// Returns all the Rust files, ignoring the other files or an error in case of problems
fn read_files(files_path: &Path) -> Result<Vec<String>, SifisError> {
    let mut vec = vec![];
    let mut first = PathBuf::new();
    first.push(files_path);
    let mut stack = vec![first];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            let paths = match fs::read_dir(path.clone()) {
                Ok(paths) => paths,
                Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
            };

            for p in paths {
                stack.push(p.unwrap().path());
            }
        } else {
            let ext = path.extension();

            if ext != None && ext.unwrap() == "rs" {
                vec.push(path.display().to_string());
            }
        }
    }
    Ok(vec)
}
/// This fuction read the content of the coveralls  json file obtain by using grcov
/// Return a HashMap with all the files arrays of covered lines using the path to the file as key
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

/// This function find the minimum space for a line i in the file
/// Tt returns the space

fn get_min_space(root: &FuncSpace, i: usize) -> FuncSpace {
    let mut min_space: FuncSpace = root.clone();
    let mut stack: Vec<FuncSpace> = vec![root.clone()];
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

/// Calculate the SIFIS plain value  for the given file(only rust language)
/// Return the value in case of success and an specif error in case of fails
fn sifis_plain(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
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

/// Calculate the SIFIS quantized value  for the given file(only rust language)
/// Return the value in case of success and an specif error in case of fails
fn sifis_quantized(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
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

// Get the code coverage in percentage
fn get_coverage_perc(covs: &[Value]) -> Result<f64, SifisError> {
    let mut tot_lines = 0.;
    let mut covered_lines = 0.;
    // count the number of covered lines
    for i in 0..covs.len() {
        let is_null = match covs.get(i) {
            Some(val) => val.is_null(),
            None => return Err(SifisError::ConversionError()),
        };

        if !is_null {
            tot_lines += 1.;
            let cov = match covs.get(i).unwrap().as_u64() {
                Some(cov) => cov,
                None => return Err(SifisError::ConversionError()),
            };
            if cov > 0 {
                covered_lines += 1.;
            }
        }
    }
    Ok(covered_lines / tot_lines)
}
/// Calculate the CRAP value  for the given file(only rust language)
/// (https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans.)
/// Return the value in case of success and an specif error in case of fails

fn crap(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
    };
    let parser = RustParser::new(data, path, None);
    let root = match metrics(&parser, path) {
        Some(root) => root,
        None => return Err(SifisError::MetricsError()),
    };
    let comp = root.metrics.cyclomatic.cyclomatic_sum();
    let cov = get_coverage_perc(covs)?;
    Ok(((comp.powf(2.)) * ((1.0 - cov).powf(3.))) + comp)
}

/// Calculate the Skunkscore value  for the given file (only rust language)
/// https://www.fastruby.io/blog/code-quality/intruducing-skunk-stink-score-calculator.html
/// In this implementation the code smeels are ignored.
/// Return the value in case of success and an specif error in case of fails

fn skunk_nosmells(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
    let complexity_factor = 25.0;
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
    };
    let parser = RustParser::new(data, path, None);
    let root = match metrics(&parser, path) {
        Some(root) => root,
        None => return Err(SifisError::MetricsError()),
    };
    let comp = root.metrics.cyclomatic.cyclomatic_sum();
    let cov = get_coverage_perc(covs)?;
    if cov == 1. {
        Ok(comp / complexity_factor)
    } else {
        Ok((comp / complexity_factor) * (100. - (100. * cov)))
    }
}

/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// It prints all the SIFIS, CRAP and SkunkScore values for all the Rust files in the folders
/// the output will be print as follows:
/// FILE       | SIFIS PLAIN | SIFIS QUANTIZED | CRAP       | SKUNKSCORE
pub fn get_metrics<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
) -> Result<(), SifisError> {
    let vec = match read_files(files_path.as_ref()) {
        Ok(vec) => vec,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_ref().display().to_string(),
            ))
        }
    };

    let file = match fs::read_to_string(json_path) {
        Ok(file) => file,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                json_path.as_ref().display().to_string(),
            ))
        }
    };
    let covs = read_json(file, files_path.as_ref().to_str().unwrap())?;
    //println!("FILE \t SIFIS PLAIN \t SIFIS QUANTIZED \t CRAP \t SKUNKSCORE");
    println!(
        "{0: <20} | {1: <20} | {2: <20} | {3: <20} | {4: <20}",
        "FILE", "SIFIS PLAIN", "SIFIS QUANTIZED", "CRAP", "SKUNKSCORE"
    );
    for path in vec {
        let arr = match covs.get(&path) {
            Some(arr) => arr.to_vec(),
            None => return Err(SifisError::HashMapError(path)),
        };
        let p = Path::new(&path);
        let sifis = sifis_plain(p, &arr)?;
        let sifis_quantized = sifis_quantized(p, &arr)?;
        let crap = crap(p, &arr)?;
        let skunk = skunk_nosmells(p, &arr)?;
        /*println!(
            "{:?} \t {:.3} \t {:.3} \t {:.3} \t {:.3}",
            p.file_name().unwrap(),
            sifis,
            sifis_quantized,
            crap,
            skunk
        );*/
        println!(
            "{0: <20} | {1: <20.3} | {2: <20.3} | {3: <20.3} | {4: <20.3}",
            p.file_name().unwrap().to_str().unwrap(),
            sifis,
            sifis_quantized,
            crap,
            skunk
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const MAIN: &str = "../rust-data-structures-main/data/main.rs";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";
    #[test]
    fn test_read_json() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        assert_eq!(covs.contains_key(SIMPLE), true);
        assert_eq!(covs.contains_key(MAIN), true);
        let vec = covs.get(SIMPLE).unwrap();
        assert_eq!(vec.len(), 12);
        let vec_main = covs.get(MAIN).unwrap();
        assert_eq!(vec_main.len(), 9);
        let value = vec.get(6).unwrap();
        assert_eq!(value, 2);
        let value_null = vec.get(1).unwrap();
        assert_eq!(value_null.is_null(), true);
    }

    #[test]
    fn test_sifis_plain() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let sifis = sifis_plain(&path, &vec).unwrap();
        assert_eq!(sifis, 24. / 10.)
    }

    #[test]
    fn test_sifis_quantized() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let sifis = sifis_quantized(&path, &vec).unwrap();
        assert_eq!(sifis, 6. / 10.)
    }

    #[test]
    fn test_crap() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let crap = crap(&path, &vec).unwrap();
        assert_eq!(crap, 5.024)
    }

    #[test]
    fn test_skunk() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let skunk = skunk_nosmells(&path, &vec).unwrap();
        assert_eq!(skunk, 6.4)
    }
}
