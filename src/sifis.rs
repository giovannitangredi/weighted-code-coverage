use crate::utility::SifisError;
use crate::utility::COMPLEXITY;
use rust_code_analysis::FuncSpace;
use serde_json::Value;

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
pub fn sifis_plain(
    root: &FuncSpace,
    covs: &[Value],
    metric: COMPLEXITY,
) -> Result<f64, SifisError> {
    let ploc = root.metrics.loc.ploc();
    let comp = match metric {
        COMPLEXITY::CYCLOMATIC => root.metrics.cyclomatic.cyclomatic_sum(),
        COMPLEXITY::COGNITIVE => root.metrics.cognitive.cognitive_sum(),
    };
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
pub fn sifis_quantized(
    root: &FuncSpace,
    covs: &[Value],
    metric: COMPLEXITY,
) -> Result<f64, SifisError> {
    let ploc = root.metrics.loc.ploc();
    let mut sum = 0.0;
    let threshold = 15.;
    //for each line find the minimun space and get complexity value then sum 1 if comp>threshold  else sum 1
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
                let min_space: FuncSpace = get_min_space(root, i);
                let comp = match metric {
                    COMPLEXITY::CYCLOMATIC => min_space.metrics.cyclomatic.cyclomatic(),
                    COMPLEXITY::COGNITIVE => min_space.metrics.cognitive.cognitive(),
                };
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
    use crate::utility::{get_root, read_json};
    use std::fs;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";
    const COMP: COMPLEXITY = COMPLEXITY::CYCLOMATIC;
    const COGN: COMPLEXITY = COMPLEXITY::COGNITIVE;

    #[test]
    fn test_sifis_plain() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let root = get_root(&path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let sifis = sifis_plain(&root, &vec, COMP).unwrap();
        assert_eq!(sifis, 24. / 10.);
        let sifis_cogn = sifis_plain(&root, &vec, COGN).unwrap();
        assert_eq!(sifis_cogn, 18. / 10.);
    }

    #[test]
    fn test_sifis_quantized() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let mut path = PathBuf::new();
        path.push(FILE);
        let root = get_root(&path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let sifis = sifis_quantized(&root, &vec, COMP).unwrap();
        assert_eq!(sifis, 6. / 10.);
        let sifis_cogn = sifis_quantized(&root, &vec, COGN).unwrap();
        assert_eq!(sifis_cogn, 6. / 10.);
    }
}
