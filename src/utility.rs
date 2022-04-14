use crate::Metrics;
use csv;
use rust_code_analysis::{get_function_spaces, guess_language, read_file, FuncSpace};
use serde_json::json;
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
    #[error("Error while guessing language")]
    LanguageError(),
    #[error("Error while writing on csv")]
    WrintingError(),
}

/// Complexity Metrics
#[derive(Copy, Debug, Clone)]
pub enum COMPLEXITY {
    CYCLOMATIC,
    COGNITIVE,
}

// This function read all  the files in the project folder
// Returns all the Rust files, ignoring the other files or an error in case of problems
pub(crate) fn read_files(files_path: &Path) -> Result<Vec<String>, SifisError> {
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

// This fuction read the content of the coveralls  json file obtain by using grcov
// Return a HashMap with all the files arrays of covered lines using the path to the file as key
pub(crate) fn read_json(
    file: String,
    prefix: &str,
) -> Result<HashMap<String, Vec<Value>>, SifisError> {
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

// Get the code coverage in percentage
pub(crate) fn get_coverage_perc(covs: &[Value]) -> Result<f64, SifisError> {
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

pub(crate) fn export_to_csv(
    csv_path: &Path,
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
) -> Result<(), SifisError> {
    let mut writer = match csv::Writer::from_path(csv_path) {
        Ok(w) => w,
        Err(_err) => return Err(SifisError::WrongFile(csv_path.display().to_string())),
    };
    match writer.write_record(&[
        "FILE",
        "SIFIS PLAIN",
        "SIFIS QUANTIZED",
        "CRAP",
        "SKUNK",
        "IGNORED",
    ]) {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    for m in metrics {
        match writer.write_record(&[
            m.file,
            format!("{:.3}", m.sifis_plain),
            format!("{:.3}", m.sifis_quantized),
            format!("{:.3}", m.crap),
            format!("{:.3}", m.skunk),
            format!("{}", false),
        ]) {
            Ok(_res) => (),
            Err(_err) => return Err(SifisError::WrintingError()),
        };
    }
    match writer.write_record(&[
        "LIST OF IGNORED FILES",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
    ]) {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    for file in files_ignored.clone() {
        match writer.write_record(&[
            file,
            format!("{:.3}", 0.),
            format!("{:.3}", 0.),
            format!("{:.3}", 0.),
            format!("{:.3}", 0.),
            format!("{}", true),
        ]) {
            Ok(_res) => (),
            Err(_err) => return Err(SifisError::WrintingError()),
        };
    }
    match writer.write_record(&[
        "TOTAL FILES IGNORED".to_string(),
        format!("{:?}", files_ignored.len()),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    ]) {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    match writer.flush() {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    Ok(())
}

// get the root FuncSpace from a file
pub(crate) fn get_root(path: &Path) -> Result<FuncSpace, SifisError> {
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
    Ok(root)
}

pub(crate) fn get_cumulative_values(metrics: &Vec<Metrics>) -> (Metrics, Metrics, Metrics) {
    let mut avg = Metrics {
        sifis_plain: 0.0,
        sifis_quantized: 0.0,
        crap: 0.0,
        skunk: 0.0,
        file: "AVG".to_string(),
    };
    let mut max = Metrics {
        sifis_plain: 0.0,
        sifis_quantized: 0.0,
        crap: 0.0,
        skunk: 0.0,
        file: "MAX".to_string(),
    };
    let mut min = Metrics {
        sifis_plain: f64::MAX,
        sifis_quantized: f64::MAX,
        crap: f64::MAX,
        skunk: f64::MAX,
        file: "MIN".to_string(),
    };
    for m in metrics {
        avg.sifis_plain += m.sifis_plain;
        avg.crap += m.crap;
        avg.skunk += m.skunk;
        avg.sifis_quantized += m.sifis_quantized;
        min.sifis_plain = min.sifis_plain.min(m.sifis_plain);
        min.crap = min.crap.min(m.crap);
        min.sifis_quantized = min.sifis_quantized.min(m.sifis_quantized);
        min.skunk = min.skunk.min(m.skunk);
        max.sifis_plain = max.sifis_plain.max(m.sifis_plain);
        max.crap = max.crap.max(m.crap);
        max.sifis_quantized = max.sifis_quantized.max(m.sifis_quantized);
        max.skunk = max.skunk.max(m.skunk);
    }
    avg.sifis_plain /= metrics.len() as f64;
    avg.crap /= metrics.len() as f64;
    avg.skunk /= metrics.len() as f64;
    avg.sifis_quantized /= metrics.len() as f64;
    (avg, min, max)
}

pub(crate) fn export_to_json(
    project_folder: &Path,
    output_path: &Path,
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
) -> Result<(), SifisError> {
    let n_files = files_ignored.len();
    let json = json!({
        "project": project_folder.display().to_string(),
        "numver_of_files_ignored": n_files,
        "metrics":metrics,
        "files_ignored":files_ignored,
    });
    let json_string = match serde_json::to_string(&json) {
        Ok(data) => data,
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    match fs::write(output_path, json_string) {
        Ok(_ok) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    const JSON: &str = "./data/data.json";
    const PREFIX: &str = "../rust-data-structures-main/";
    const MAIN: &str = "../rust-data-structures-main/data/main.rs";
    const SIMPLE: &str = "../rust-data-structures-main/data/simple_main.rs";

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
