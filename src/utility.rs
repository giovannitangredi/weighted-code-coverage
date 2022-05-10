use crate::Metrics;
use csv;
use rust_code_analysis::{get_function_spaces, guess_language, read_file, FuncSpace};
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use std::collections::*;
use std::ffi::OsStr;
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
    #[error("Error during concurrency")]
    ConcurrentError(),
    #[error("Json Tpye is not supported!")]
    TypeError,
}

/// Complexity Metrics
#[derive(Copy, Debug, Clone)]
pub enum COMPLEXITY {
    CYCLOMATIC,
    COGNITIVE,
}
// check all possible valid extentions
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

            if ext.is_some() && check_ext(ext.unwrap()) {
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
        if cfg!(windows) {
            name += x["name"].as_str().unwrap().replace('/', "\\").as_str();
        } else {
            name += x["name"].as_str().unwrap();
        }
        let value = match x["coverage"].as_array() {
            Some(value) => value.to_vec(),
            None => return Err(SifisError::ConversionError()),
        };
        covs.insert(name.to_string(), value);
    }
    Ok(covs)
}
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub(crate) struct Covdir {
    pub(crate)  name: String,
    pub(crate)  arr: Vec<Value>,
    pub(crate)  coverage: f64,
}
// This fuction read the content of the coveralls  json file obtain by using grcov
// Return a HashMap with all the files arrays of covered lines using the path to the file as key
pub(crate) fn read_json_covdir(
    file: String,
    map_prefix: &str,
) -> Result<HashMap<String, Covdir>, SifisError> {
    let val: Map<String, Value> = match serde_json::from_str(file.as_str()) {
        Ok(val) => val,
        Err(_err) => return Err(SifisError::ReadingJSONError()),
    };
    let mut res: HashMap<String, Covdir> = HashMap::<String, Covdir>::new();
    let mut stack = Vec::<(Map<String, Value>, String)>::new();
    stack.push((val["children"].as_object().unwrap().clone(), "".to_string()));
    let covdir = Covdir {
        name: val["name"].as_str().unwrap().to_string(),
        arr: vec![],
        coverage: val["coveragePercent"].as_f64().unwrap(),
    };
    res.insert("PROJECT_ROOT".to_string(), covdir);
    while let Some((val, prefix)) = stack.pop() {
        for (key, value) in val {
            if value["children"].is_object() {
                if prefix == "" {
                    stack.push((
                        value["children"].as_object().unwrap().clone(),
                        prefix.to_owned() + key.as_str(),
                    ));
                } else {
                    stack.push((
                        value["children"].as_object().unwrap().clone(),
                        prefix.to_owned() + "/" + key.as_str(),
                    ));
                }
            }
            let name = value["name"].as_str().unwrap().to_string();
            let path = Path::new(&name);
            let ext = path.extension();

            if ext.is_some() && check_ext(ext.unwrap()) {
                let covdir = Covdir {
                    name: name,
                    arr: value["coverage"].as_array().unwrap().to_vec(),
                    coverage: value["coveragePercent"].as_f64().unwrap(),
                };
                let name_path = format!("{}/{}", prefix, key);
                res.insert(map_prefix.to_owned() + name_path.as_str(), covdir);
            }
        }
    }
    Ok(res)
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

// Get the code coverage in percentage
pub(crate) fn get_covered_lines(covs: &[Value]) -> Result<(f64,f64), SifisError> {
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
    Ok((covered_lines , tot_lines))
}

pub(crate) fn export_to_csv(
    csv_path: &Path,
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    project_coverage : f64,
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
        "IS COMPLEX",
        "FILE PATH",
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
            format!("{}", m.is_complex),
            m.file_path,
        ]) {
            Ok(_res) => (),
            Err(_err) => return Err(SifisError::WrintingError()),
        };
    }
    match writer.write_record(&[
        "PROJECT_COVERAGE",
        format!("{:.3}", project_coverage).as_str(),
        "-",
        "-",
        "-",
        "-",
        "-",
        "-",
    ]) {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    match writer.write_record(&[
        "LIST OF COMPLEX FILES",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
    ]) {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    for m in complex_files.clone() {
        match writer.write_record(&[
            m.file,
            format!("{:.3}", m.sifis_plain),
            format!("{:.3}", m.sifis_quantized),
            format!("{:.3}", m.crap),
            format!("{:.3}", m.skunk),
            format!("{}", false),
            format!("{}", m.is_complex),
            m.file_path,
        ]) {
            Ok(_res) => (),
            Err(_err) => return Err(SifisError::WrintingError()),
        };
    }
    match writer.write_record(&[
        "TOTAL COMPLEX FILES".to_string(),
        format!("{:?}", complex_files.len()),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    ]) {
        Ok(_res) => (),
        Err(_err) => return Err(SifisError::WrintingError()),
    };
    match writer.write_record(&[
        "LIST OF IGNORED FILES",
        "----------",
        "----------",
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
            "-".to_string(),
            "-".to_string(),
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

pub(crate) fn check_complexity(
    sifis_plain: f64,
    sifis_quantized: f64,
    crap: f64,
    skunk: f64,
) -> bool {
    if sifis_plain > 35. {
        return true;
    }
    if sifis_quantized > 1.5 {
        return true;
    }
    if crap > 35. {
        return true;
    }
    if skunk > 20. {
        return true;
    }
    false
}
pub(crate) fn get_cumulative_values(metrics: &Vec<Metrics>) -> (Metrics, Metrics,Metrics, Vec<Metrics>) {
    let mut avg = Metrics {
        sifis_plain: 0.0,
        sifis_quantized: 0.0,
        crap: 0.0,
        skunk: 0.0,
        file: "AVG".to_string(),
        file_path: "-".to_string(),
        is_complex: false,
        coverage: 0.0
    };
    let mut min = Metrics {
        sifis_plain: 0.0,
        sifis_quantized: 0.0,
        crap: 0.0,
        skunk: 0.0,
        file: "MIN".to_string(),
        file_path: "-".to_string(),
        is_complex: false,
        coverage: 0.0
    };
    let mut max = Metrics {
        sifis_plain: 0.0,
        sifis_quantized: 0.0,
        crap: 0.0,
        skunk: 0.0,
        file: "MAX".to_string(),
        file_path: "-".to_string(),
        is_complex: false,
        coverage: 0.0
    };
    let mut complex_files = Vec::<Metrics>::new();
    for m in metrics {
        avg.sifis_plain += m.sifis_plain;
        avg.crap += m.crap;
        avg.skunk += m.skunk;
        avg.sifis_quantized += m.sifis_quantized;
        avg.coverage += m.coverage*100.;
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
    }
    avg.sifis_plain /= metrics.len() as f64;
    avg.crap /= metrics.len() as f64;
    avg.skunk /= metrics.len() as f64;
    avg.sifis_quantized /= metrics.len() as f64;
    avg.coverage /= metrics.len() as f64;
    (avg, max, min, complex_files)
}

pub(crate) fn export_to_json(
    project_folder: &Path,
    output_path: &Path,
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    project_coverage: f64,
) -> Result<(), SifisError> {
    let n_files = files_ignored.len();
    let number_of_complex_files = complex_files.len();
    let json = json!({
        "project": project_folder.display().to_string(),
        "number_of_files_ignored": n_files,
        "number_of_complex_files": number_of_complex_files,
        "metrics":metrics,
        "files_ignored":files_ignored,
        "complex_files": complex_files,
        "project_coverage" : project_coverage,
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
