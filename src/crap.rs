use crate::utility::{get_coverage_perc, SifisError ,COMPLEXITY};
use rust_code_analysis::{get_function_spaces, guess_language, read_file};
use serde_json::Value;
use std::path::*;

/// Calculate the CRAP value  for the given file(only rust language)
/// (https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans.)
/// Return the value in case of success and an specif error in case of fails

pub fn crap(path: &Path, covs: &[Value], metric : COMPLEXITY) -> Result<f64, SifisError> {
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
    let comp = match metric {
        COMPLEXITY::CYCLOMATIC => root.metrics.cyclomatic.cyclomatic_sum(),
        COMPLEXITY::COGNITIVE => root.metrics.cognitive.cognitive_sum()
    };
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
    const COMP : COMPLEXITY = COMPLEXITY::CYCLOMATIC;
    const COGN : COMPLEXITY = COMPLEXITY::COGNITIVE;

    #[test]
    fn test_crap() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let crap_cy = crap(&path, &vec,COMP).unwrap();
        assert_eq!(crap_cy, 5.024);
        let crap_cogn = crap(&path, &vec,COGN).unwrap();
        assert_eq!(crap_cogn, 3.576);
    }
}
