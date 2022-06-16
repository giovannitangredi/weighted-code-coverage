use rust_code_analysis::FuncSpace;
use serde_json::Value;

use crate::error::*;
use crate::utility::Complexity;

const THRESHOLD: f64 = 15.;
// This function find the minimum space for a line i in the file
// It returns the space
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

// Calculate the SIFIS plain value  for the given file
// Return the value in case of success and an specif error in case of fails
pub(crate) fn sifis_plain(
    root: &FuncSpace,
    covs: &[Value],
    metric: Complexity,
    is_covdir: bool,
) -> Result<(f64, f64)> {
    let ploc = root.metrics.loc.ploc();
    let comp = match metric {
        Complexity::Cyclomatic => root.metrics.cyclomatic.cyclomatic_sum(),
        Complexity::Cognitive => root.metrics.cognitive.cognitive_sum(),
    };
    let sum = covs.iter().try_fold(0., |acc, line| -> Result<f64> {
        // Check if the line is null
        let is_null = if is_covdir {
            line.as_i64().ok_or(Error::ConversionError())? == -1
        } else {
            line.is_null()
        };
        let sum;
        if !is_null {
            // If the line is not null and is covered (cov>0) the add the complexity  to the sum
            let cov = line.as_u64().ok_or(Error::ConversionError())?;
            if cov > 0 {
                sum = acc + comp;
            } else {
                sum = acc;
            }
        } else {
            sum = acc;
        }
        Ok(sum)
    })?;
    Ok((sum / ploc, sum))
}

// Calculate the SIFIS quantized value  for the given file
// Return the value in case of success and an specif error in case of fails
// If the complexity of the block/file is 0 the value if sifis quantized is the coverage of the file
pub(crate) fn sifis_quantized(
    root: &FuncSpace,
    covs: &[Value],
    metric: Complexity,
    is_covdir: bool,
) -> Result<(f64, f64)> {
    let ploc = root.metrics.loc.ploc();
    let sum =
    //For each line find the minimum space and get complexity value then sum 1 if comp>threshold  else sum 1
        covs.iter()
            .enumerate()
            .try_fold(0., |acc, (i, line)| -> Result<f64> {
                // Check if the line is null
                let is_null = if is_covdir {
                    line.as_i64().ok_or(Error::ConversionError())? == -1
                } else {
                    line.is_null()
                };
                let sum;
                if !is_null {
                    // Get line
                    let cov = line.as_u64().ok_or(Error::ConversionError())?;
                    if cov > 0 {
                        // If the line is covered get the space of the line and then check if the complexity is below the threshold
                        let min_space: FuncSpace = get_min_space(root, i);
                        let comp = match metric {
                            Complexity::Cyclomatic => min_space.metrics.cyclomatic.cyclomatic(),
                            Complexity::Cognitive => min_space.metrics.cognitive.cognitive(),
                        };
                        if comp > THRESHOLD {
                            sum = acc + 2.;
                        } else {
                            sum = acc + 1.;
                        }
                    } else {
                        sum = acc;
                    }
                } else {
                    sum = acc;
                }
                Ok(sum)
            })?;
    Ok((sum / ploc, sum))
}

pub(crate) fn sifis_plain_function(
    space: &FuncSpace,
    covs: &[Value],
    metric: Complexity,
    is_covdir: bool,
) -> Result<(f64, f64)> {
    let ploc = space.metrics.loc.ploc();
    let comp = match metric {
        Complexity::Cyclomatic => space.metrics.cyclomatic.cyclomatic_sum(),
        Complexity::Cognitive => space.metrics.cognitive.cognitive_sum(),
    };
    let sum = covs
        .iter()
        .enumerate()
        .try_fold(0., |acc, (i, line)| -> Result<f64> {
            // Check if the line is null
            let is_null = if is_covdir {
                line.as_i64().ok_or(Error::ConversionError())? == -1
            } else {
                line.is_null()
            };
            let sum;
            if !is_null && i >= space.start_line - 1 && i < space.end_line {
                // If the line is not null and is covered (cov>0) the add the complexity  to the sum
                let cov = line.as_u64().ok_or(Error::ConversionError())?;
                if cov > 0 {
                    sum = acc + comp;
                } else {
                    sum = acc;
                }
            } else {
                sum = acc;
            }
            Ok(sum)
        })?;
    Ok((sum / ploc, sum))
}

pub(crate) fn sifis_quantized_function(
    space: &FuncSpace,
    covs: &[Value],
    metric: Complexity,
    is_covdir: bool,
) -> Result<(f64, f64)> {
    let ploc = space.metrics.loc.ploc();
    let sum =
    //For each line find the minimum space and get complexity value then sum 1 if comp>threshold  else sum 1
        covs.iter()
            .enumerate()
            .try_fold(0., |acc, (i, line)| -> Result<f64> {
                // Check if the line is null
                let is_null = if is_covdir {
                    line.as_i64().ok_or(Error::ConversionError())? == -1
                } else {
                    line.is_null()
                };
                let sum;
                if !is_null && i>= space.start_line-1 && i< space.end_line {
                    // Get line
                    let cov = line.as_u64().ok_or(Error::ConversionError())?;
                    if cov > 0 {
                        // If the line is covered get the space of the line and then check if the complexity is below the threshold
                        let min_space: FuncSpace = get_min_space(space, i);
                        let comp = match metric {
                            Complexity::Cyclomatic => min_space.metrics.cyclomatic.cyclomatic(),
                            Complexity::Cognitive => min_space.metrics.cognitive.cognitive(),
                        };
                        if comp > THRESHOLD {
                            sum = acc + 2.;
                        } else {
                            sum = acc + 1.;
                        }
                    } else {
                        sum = acc;
                    }
                } else {
                    sum = acc;
                }
                Ok(sum)
            })?;
    Ok((sum / ploc, sum))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::utility::{get_root, read_json};
    use std::fs;

    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const FILE: &str = "./data/simple_main.rs";
    const COMP: Complexity = Complexity::Cyclomatic;
    const COGN: Complexity = Complexity::Cognitive;

    #[test]
    fn test_sifis_plain_cyclomatic() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis, _) = sifis_plain(&root, &vec, COMP, false).unwrap();
        assert_eq!(sifis, 24. / 10.);
    }

    #[test]
    fn test_sifis_plain_cognitive() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis_cogn, _) = sifis_plain(&root, &vec, COGN, false).unwrap();
        assert_eq!(sifis_cogn, 18. / 10.);
    }

    #[test]
    fn test_sifis_quantized_cyclomatic() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis, _) = sifis_quantized(&root, &vec, COMP, false).unwrap();
        assert_eq!(sifis, 6. / 10.);
    }

    #[test]
    fn test_sifis_quantized_cognitive() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis_cogn, _) = sifis_quantized(&root, &vec, COGN, false).unwrap();
        assert_eq!(sifis_cogn, 6. / 10.);
    }

    #[test]
    fn test_sifis_plain_cyclomatic_function() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis, _) = sifis_plain_function(&root, &vec, COMP, false).unwrap();
        assert_eq!(sifis, 24. / 10.);
    }

    #[test]
    fn test_sifis_plain_cognitive_function() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis_cogn, _) = sifis_plain_function(&root, &vec, COGN, false).unwrap();
        assert_eq!(sifis_cogn, 18. / 10.);
    }

    #[test]
    fn test_sifis_quantized_cyclomatic_function() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis, _) = sifis_quantized_function(&root, &vec, COMP, false).unwrap();
        assert_eq!(sifis, 6. / 10.);
    }

    #[test]
    fn test_sifis_quantized_cognitive_function() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        let path = Path::new(FILE);
        let root = get_root(path).unwrap();
        let vec = covs.get(SIMPLE).unwrap().to_vec();
        let (sifis_cogn, _) = sifis_quantized_function(&root, &vec, COGN, false).unwrap();
        assert_eq!(sifis_cogn, 6. / 10.);
    }
}
