use crate::utility::{get_coverage_perc, SifisError};
use rust_code_analysis::{metrics, read_file, ParserTrait, RustParser};
use serde_json::Value;
use std::path::*;

/// Calculate the CRAP value  for the given file(only rust language)
/// (https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans.)
/// Return the value in case of success and an specif error in case of fails

pub fn crap(path: &Path, covs: &[Value]) -> Result<f64, SifisError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::read_json;
    use std::fs;
    use std::path::PathBuf;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";

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
}
