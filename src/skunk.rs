use crate::utility::{get_coverage_perc, SifisError, COMPLEXITY};
use rust_code_analysis::{get_function_spaces, guess_language, read_file};
use serde_json::Value;
use std::path::*;

/// Calculate the Skunkscore value  for the given file (only rust language)
/// https://www.fastruby.io/blog/code-quality/intruducing-skunk-stink-score-calculator.html
/// In this implementation the code smeels are ignored.
/// Return the value in case of success and an specif error in case of fails
pub fn skunk_nosmells(path: &Path, covs: &[Value], metric : COMPLEXITY) -> Result<f64, SifisError> {
    let complexity_factor = 25.0;
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
    if cov == 1. {
        Ok(comp / complexity_factor)
    } else {
        Ok((comp / complexity_factor) * (100. - (100. * cov)))
    }
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
    fn test_skunk() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let skunk = skunk_nosmells(&path, &vec,COMP).unwrap();
        assert_eq!(skunk, 6.4);
        let skunk_cogn = skunk_nosmells(&path, &vec,COGN).unwrap();
        assert_eq!(skunk_cogn, 4.8);
    }
}
