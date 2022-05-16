use crate::utility::{get_coverage_perc, SifisError, COMPLEXITY};
use rust_code_analysis::FuncSpace;
use serde_json::Value;

/// Calculate the Skunkscore value  for the given file (only rust language)
/// https://www.fastruby.io/blog/code-quality/intruducing-skunk-stink-score-calculator.html
/// In this implementation the code smeels are ignored.
/// Return the value in case of success and an specif error in case of fails
pub fn skunk_nosmells(
    root: &FuncSpace,
    covs: &[Value],
    metric: COMPLEXITY,
    coverage: Option<f64>,
) -> Result<f64, SifisError> {
    let complexity_factor = 25.0;
    let comp = match metric {
        COMPLEXITY::CYCLOMATIC => root.metrics.cyclomatic.cyclomatic_sum(),
        COMPLEXITY::COGNITIVE => root.metrics.cognitive.cognitive_sum(),
    };
    let cov = if let Some(coverage) = coverage {
        coverage / 100.0
    } else {
        match get_coverage_perc(covs) {
            Ok(cov) => cov,
            Err(err) => return Err(err),
        }
    };
    if cov == 1. {
        Ok(comp / complexity_factor)
    } else {
        Ok((comp / complexity_factor) * (100. - (100. * cov)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::{get_root, read_json};
    use std::fs;
    use std::path::PathBuf;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";
    const COMP: COMPLEXITY = COMPLEXITY::CYCLOMATIC;
    const COGN: COMPLEXITY = COMPLEXITY::COGNITIVE;

    #[test]
    fn test_skunk() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let root = get_root(&path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let skunk = skunk_nosmells(&root, &vec, COMP).unwrap();
        assert_eq!(skunk, 6.4);
        let skunk_cogn = skunk_nosmells(&root, &vec, COGN).unwrap();
        assert_eq!(skunk_cogn, 4.8);
    }
}
