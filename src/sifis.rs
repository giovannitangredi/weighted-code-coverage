use crate::utility::SifisError;
use rust_code_analysis::{get_function_spaces, guess_language, read_file, FuncSpace};
use serde_json::Value;
use std::path::*;

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
pub fn sifis_plain(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
    };
    let lang = match guess_language(&data, path).0 {
        Some(lang) => lang,
        None => return Err(SifisError::LanguageError()),
    };
    let space = match get_function_spaces(&lang, data, path, None) {
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
pub fn sifis_quantized(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
    let data = match read_file(path) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrongFile(path.display().to_string())),
    };
    let lang = match guess_language(&data, path).0 {
        Some(lang) => lang,
        None => return Err(SifisError::LanguageError()),
    };
    let root = match get_function_spaces(&lang, data, path, None) {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::utility::read_json;
    use std::fs;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";

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
}
