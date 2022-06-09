use rust_code_analysis::FuncSpace;
use serde_json::Value;

use crate::utility::{get_coverage_perc, Complexity, SifisError};

// Calculate the CRAP value  for the given file
// (https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans.)
// Return the value in case of success and an specif error in case of fails
pub(crate) fn crap(
    root: &FuncSpace,
    covs: &[Value],
    metric: Complexity,
    coverage: Option<f64>,
) -> Result<f64, SifisError> {
    let comp = match metric {
        Complexity::Cyclomatic => root.metrics.cyclomatic.cyclomatic_sum(),
        Complexity::Cognitive => root.metrics.cognitive.cognitive_sum(),
    };
    let cov = if let Some(coverage) = coverage {
        coverage / 100.0
    } else {
        get_coverage_perc(covs)?
    };
    Ok(((comp.powf(2.)) * ((1.0 - cov).powf(3.))) + comp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::{get_root, read_json};
    use std::fs;
    use std::path::Path;

    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";
    const COMP: Complexity = Complexity::Cyclomatic;
    const COGN: Complexity = Complexity::Cognitive;

    #[test]
    fn test_crap_cyclomatic() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let crap_cy = crap(&root, &vec, COMP, None).unwrap();
        assert_eq!(crap_cy, 5.024);
    }

    #[test]
    fn test_crap_cognitive() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let crap_cogn = crap(&root, &vec, COGN, None).unwrap();
        assert_eq!(crap_cogn, 3.576);
    }
}
