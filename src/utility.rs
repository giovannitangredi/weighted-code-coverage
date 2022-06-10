use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::*;

use arg_enum_proc_macro::ArgEnum;
use rust_code_analysis::{get_function_spaces, guess_language, read_file, FuncSpace};
use serde_json::Map;
use serde_json::Value;
use tracing::debug;

use crate::error::Error;
use crate::Config;
use crate::Metrics;

const COMPLEXITY_FACTOR: f64 = 25.0;

/// Complexity Metrics
#[derive(ArgEnum, Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Complexity {
    /// Cyclomatic metric.
    #[arg_enum(name = "cyclomatic")]
    Cyclomatic,
    /// Cognitive metric.
    #[arg_enum(name = "cognitive")]
    Cognitive,
}
impl Complexity {
    /// Default Complexity format.
    pub const fn default() -> &'static str {
        "cyclomatic"
    }
}

/// JSONs format available
#[derive(ArgEnum, Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum JsonFormat {
    /// Cyclomatic metric.
    #[arg_enum(name = "covdir")]
    Covdir,
    /// Cognitive metric.
    #[arg_enum(name = "coveralls")]
    Coveralls,
}
impl JsonFormat {
    /// Default output format.
    pub const fn default() -> &'static str {
        "coveralls"
    }
}

// Check all possible valid extensions
#[inline(always)]
fn check_ext(ext: &OsStr) -> bool {
    ext == "rs"
        || ext == "cpp"
        || ext == "c"
        || ext == "js"
        || ext == "java"
        || ext == "py"
        || ext == "tsx"
        || ext == "ts"
        || ext == "jsm"
}

// This function read all  the files in the project folder
// Returns all the source files, ignoring the other files or an error in case of problems
pub(crate) fn read_files(files_path: &Path) -> Result<Vec<String>, Error> {
    debug!("REading files in project folder: {:?}", files_path);
    let mut vec = vec![];
    let mut first = PathBuf::new();
    first.push(files_path);
    let mut stack = vec![first];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            let mut paths = fs::read_dir(&path)?;
            paths.try_for_each(|p| -> Result<(), Error> {
                let pa = p?.path();
                stack.push(pa);
                Ok(())
            })?;
        } else {
            let ext = path.extension();

            if ext.is_some() && check_ext(ext.ok_or(Error::PathConversionError())?) {
                vec.push(path.display().to_string().replace('\\', "/"));
            }
        }
    }
    Ok(vec)
}

// This function read the content of the coveralls  json file obtain by using grcov
// Return a HashMap with all the files arrays of covered lines using the path to the file as key
pub(crate) fn read_json(file: String, prefix: &str) -> Result<HashMap<String, Vec<Value>>, Error> {
    debug!("Reading coveralls json...");
    let val: Value = serde_json::from_str(file.as_str())?;
    let vec = val["source_files"]
        .as_array()
        .ok_or(Error::ReadingJSONError())?;
    let mut covs = HashMap::<String, Vec<Value>>::new();
    vec.iter().try_for_each(|x| -> Result<(), Error> {
        let name = Path::new(prefix).join(x["name"].as_str().ok_or(Error::PathConversionError())?);
        let value = x["coverage"]
            .as_array()
            .ok_or(Error::ConversionError())?
            .to_vec();
        covs.insert(name.display().to_string().replace('\\', "/"), value);
        Ok(())
    })?;
    Ok(covs)
}

// Struct used for covdir json parsing
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub(crate) struct Covdir {
    pub(crate) name: String,
    pub(crate) arr: Vec<Value>,
    pub(crate) coverage: f64,
}

// This function read the content of the coveralls  json file obtain by using grcov
// Return a HashMap with all the files arrays of covered lines using the path to the file as key
pub(crate) fn read_json_covdir(
    file: String,
    map_prefix: &str,
) -> Result<HashMap<String, Covdir>, Error> {
    debug!("Reading covdir json...");
    let val: Map<String, Value> = serde_json::from_str(file.as_str())?;
    let mut res: HashMap<String, Covdir> = HashMap::<String, Covdir>::new();
    let mut stack = vec![(
        val["children"]
            .as_object()
            .ok_or(Error::ConversionError())?,
        "".to_string(),
    )];
    let covdir = Covdir {
        name: val["name"]
            .as_str()
            .ok_or(Error::ConversionError())?
            .to_string(),
        arr: vec![],
        coverage: val["coveragePercent"]
            .as_f64()
            .ok_or(Error::ConversionError())?,
    };
    res.insert("PROJECT_ROOT".to_string(), covdir);
    while let Some((val, prefix)) = stack.pop() {
        val.iter()
            .try_for_each(|(key, value)| -> Result<(), Error> {
                if value["children"].is_object() {
                    if prefix.is_empty() {
                        stack.push((
                            value["children"]
                                .as_object()
                                .ok_or(Error::ConversionError())?,
                            prefix.to_owned() + key.as_str(),
                        ));
                    } else {
                        let slash = if cfg!(windows) { "\\" } else { "/" };
                        stack.push((
                            value["children"]
                                .as_object()
                                .ok_or(Error::ConversionError())?,
                            prefix.to_owned() + slash + key.as_str(),
                        ));
                    }
                }
                let name = value["name"]
                    .as_str()
                    .ok_or(Error::ConversionError())?
                    .to_string();
                let path = Path::new(&name);
                let ext = path.extension();

                if ext.is_some() && check_ext(ext.ok_or(Error::PathConversionError())?) {
                    let covdir = Covdir {
                        name,
                        arr: value["coverage"]
                            .as_array()
                            .ok_or(Error::ConversionError())?
                            .to_vec(),
                        coverage: value["coveragePercent"]
                            .as_f64()
                            .ok_or(Error::ConversionError())?,
                    };
                    let name_path = format!("{}/{}", prefix, key);
                    res.insert(map_prefix.to_owned() + name_path.as_str(), covdir);
                }
                Ok(())
            })?;
    }
    Ok(res)
}

// Get the code coverage in percentage
pub(crate) fn get_coverage_perc(covs: &[Value]) -> Result<f64, Error> {
    // Count the number of covered lines
    let (tot_lines, covered_lines) =
        covs.iter()
            .try_fold((0., 0.), |acc, line| -> Result<(f64, f64), Error> {
                let is_null = line.is_null();
                let sum;
                if !is_null {
                    let cov = line.as_u64().ok_or(Error::ConversionError())?;
                    if cov > 0 {
                        sum = (acc.0 + 1., acc.1 + 1.);
                    } else {
                        sum = (acc.0 + 1., acc.1);
                    }
                } else {
                    sum = (acc.0, acc.1);
                }
                Ok(sum)
            })?;
    Ok(covered_lines / tot_lines)
}

// Get the code coverage in percentage
pub(crate) fn get_covered_lines(covs: &[Value]) -> Result<(f64, f64), Error> {
    // Count the number of covered lines
    let (tot_lines, covered_lines) =
        covs.iter()
            .try_fold((0., 0.), |acc, line| -> Result<(f64, f64), Error> {
                let is_null = line.is_null();
                let sum;
                if !is_null {
                    let cov = line.as_u64().ok_or(Error::ConversionError())?;
                    if cov > 0 {
                        sum = (acc.0 + 1., acc.1 + 1.);
                    } else {
                        sum = (acc.0 + 1., acc.1);
                    }
                } else {
                    sum = (acc.0, acc.1);
                }
                Ok(sum)
            })?;
    Ok((covered_lines, tot_lines))
}

// Get the root FuncSpace from a file
pub(crate) fn get_root(path: &Path) -> Result<FuncSpace, Error> {
    let data = read_file(path)?;
    let lang = guess_language(&data, path)
        .0
        .ok_or(Error::LanguageError())?;
    debug!("{:?} is written in {:?}", path, lang);
    let root = get_function_spaces(&lang, data, path, None).ok_or(Error::MetricsError())?;
    Ok(root)
}

// Check complexity of a metric
// Return true if al least one metric exceed a threshold , false otherwise
#[inline(always)]
pub(crate) fn check_complexity(
    sifis_plain: f64,
    sifis_quantized: f64,
    crap: f64,
    skunk: f64,
    thresholds: &[f64],
) -> bool {
    sifis_plain > thresholds[0]
        || sifis_quantized > thresholds[1]
        || crap > thresholds[2]
        || skunk > thresholds[3]
}

// Get AVG MIN MAx and the list of all complex files
pub(crate) fn get_cumulative_values(
    metrics: &Vec<Metrics>,
) -> (Metrics, Metrics, Metrics, Vec<Metrics>) {
    let mut avg = Metrics::avg();
    let mut min = Metrics::min();
    let mut max = Metrics::max();
    let mut complex_files = Vec::<Metrics>::new();
    let (sifis, sifisq, crap, skunk, cov) =
        metrics.iter().fold((0.0, 0.0, 0.0, 0.0, 0.0), |acc, m| {
            max.sifis_plain = max.sifis_plain.max(m.sifis_plain);
            max.sifis_quantized = max.sifis_quantized.max(m.sifis_quantized);
            max.crap = max.crap.max(m.crap);
            max.skunk = max.skunk.max(m.skunk);
            min.sifis_plain = min.sifis_plain.min(m.sifis_plain);
            min.sifis_quantized = min.sifis_quantized.min(m.sifis_quantized);
            min.crap = min.crap.min(m.crap);
            min.skunk = min.skunk.min(m.skunk);
            if m.is_complex {
                complex_files.push(m.clone());
            }
            (
                acc.0 + m.sifis_plain,
                acc.1 + m.sifis_quantized,
                acc.2 + m.crap,
                acc.3 + m.skunk,
                acc.4 + m.coverage,
            )
        });
    avg.sifis_plain = sifis / metrics.len() as f64;
    avg.crap = crap / metrics.len() as f64;
    avg.skunk = skunk / metrics.len() as f64;
    avg.sifis_quantized = sifisq / metrics.len() as f64;
    avg.coverage = cov / metrics.len() as f64;
    (avg, max, min, complex_files)
}

// Calculate SIFIS PLAIN , SIFIS QUANTIZED, CRA and SKUNKSCORE for the entire project
// Using the sum values computed before
pub(crate) fn get_project_metrics(project_coverage: f64, cfg: &Config) -> Result<Metrics, Error> {
    let sifis_plain_sum = *cfg.sifis_plain_sum.lock()?;
    let sifis_quantized_sum = *cfg.sifis_quantized_sum.lock()?;
    let ploc_sum = *cfg.ploc_sum.lock()?;
    let comp_sum = *cfg.comp_sum.lock()?;
    Ok(Metrics {
        sifis_plain: sifis_plain_sum / ploc_sum,
        sifis_quantized: sifis_quantized_sum / ploc_sum,
        crap: ((comp_sum.powf(2.)) * ((1.0 - project_coverage / 100.).powf(3.))) + comp_sum,
        skunk: (comp_sum / COMPLEXITY_FACTOR) * (100. - (project_coverage)),
        file: "PROJECT".to_string(),
        file_path: "-".to_string(),
        is_complex: false,
        coverage: project_coverage,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";
    const MAIN: &str = "../rust-data-structures-main/data/main.rs";

    #[test]
    fn test_read_json() {
        let file = fs::read_to_string(JSON).unwrap();
        let covs = read_json(file, PREFIX).unwrap();
        assert!(covs.contains_key(SIMPLE));
        assert!(covs.contains_key(MAIN));
        let vec = covs.get(SIMPLE).unwrap();
        assert_eq!(vec.len(), 12);
        let vec_main = covs.get(MAIN).unwrap();
        assert_eq!(vec_main.len(), 9);
        let value = vec.get(6).unwrap();
        assert_eq!(value, 2);
        let value_null = vec.get(1).unwrap();
        assert!(value_null.is_null());
    }
}
