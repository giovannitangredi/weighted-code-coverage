pub mod crap;
pub mod sifis;
pub mod skunk;
pub mod utility;

use crate::crap::crap;
use crate::sifis::{sifis_plain, sifis_quantized};
use crate::skunk::skunk_nosmells;
use crate::utility::*;
use crossbeam::channel::{unbounded, Receiver};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::*;
use std::thread;

/// Struct with all the metrics computed for a single file
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Metrics {
    sifis_plain: f64,
    sifis_quantized: f64,
    crap: f64,
    skunk: f64,
    file: String,
    file_path: String,
    is_complex: bool,
    coverage: f64,
}

type Output = (Vec<Metrics>, Vec<String>, Vec<Metrics>, f64);
/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// It prints all the SIFIS, CRAP and SkunkScore values for all the files in the folders
/// the output will be print as follows:
/// FILE       | SIFIS PLAIN | SIFIS QUANTIZED | CRAP       | SKUNKSCORE
/// if the a file is not found in the json that files will be skipped
pub fn get_metrics_output(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
) -> Result<(), SifisError> {
    println!(
        "{0: <20} | {1: <20} | {2: <20} | {3: <20} | {4: <20} | {5: <20} | {6: <30}",
        "FILE", "SIFIS PLAIN", "SIFIS QUANTIZED", "CRAP", "SKUNKSCORE", "IS_COMPLEX", "FILE PATH"
    );
    for m in metrics {
        println!(
            "{0: <20} | {1: <20.3} | {2: <20.3} | {3: <20.3} | {4: <20.3} | {5: <20} | {6: <30}",
            m.file, m.sifis_plain, m.sifis_quantized, m.crap, m.skunk, m.is_complex, m.file_path
        );
    }
    println!("FILES IGNORED: {}", files_ignored.len());
    println!("COMPLEX FILES: {}", complex_files.len());
    Ok(())
}

/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// if the a file is not found in the json that files will be skipped
/// It returns a tuple with a vector with all the metrics for a file and the comulative values and a vector with the list of all ignored files
pub fn get_metrics<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: COMPLEXITY,
) -> Result<Output, SifisError> {
    let vec = match read_files(files_path.as_ref()) {
        Ok(vec) => vec,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_ref().display().to_string(),
            ))
        }
    };
    let mut covered_lines = 0.;
    let mut tot_lines = 0.;
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    let file = match fs::read_to_string(json_path) {
        Ok(file) => file,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                json_path.as_ref().display().to_string(),
            ))
        }
    };
    let covs = read_json(file, files_path.as_ref().to_str().unwrap())?;
    for path in vec {
        let p = Path::new(&path);
        let file = p.file_name().unwrap().to_str().unwrap().to_string();
        let arr = match covs.get(&path) {
            Some(arr) => arr.to_vec(),
            None => {
                files_ignored.push(file);
                continue;
            }
        };
        let (_covered_lines, _tot_lines) = get_covered_lines(&arr)?;
        covered_lines += _covered_lines;
        tot_lines += _tot_lines;
        let root = get_root(p)?;
        let sifis_plain = sifis_plain(&root, &arr, metric, false)?;
        let sifis_quantized = sifis_quantized(&root, &arr, metric, false)?;
        let crap = crap(&root, &arr, metric, None)?;
        let skunk = skunk_nosmells(&root, &arr, metric, None)?;
        let file_path = path
            .clone()
            .split_off(files_path.as_ref().to_str().unwrap().len());
        let is_complex = check_complexity(sifis_plain, sifis_quantized, crap, skunk);
        res.push(Metrics {
            sifis_plain,
            sifis_quantized,
            crap,
            skunk,
            file,
            file_path,
            is_complex,
            coverage: get_coverage_perc(&arr).unwrap(),
        });
    }
    let (avg, max, min, complex_files) = get_cumulative_values(&res);
    res.push(avg);
    res.push(max);
    res.push(min);
    let project_coverage = covered_lines / tot_lines;
    Ok((res, files_ignored, complex_files, project_coverage))
}

struct JobItem {
    chunck: Vec<String>,
    covs: HashMap<String, Vec<Value>>,
    metric: COMPLEXITY,
    prefix: usize,
}

type JobReceiver = Receiver<Option<JobItem>>;

fn consumer(receiver: JobReceiver) -> Result<(Vec<Metrics>, Vec<String>, f64, f64), SifisError> {
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    let mut all_cov_lines = 0.;
    let mut all_tot_lines = 0.;
    while let Ok(job) = receiver.recv() {
        if job.is_none() {
            break;
        }
        // Cannot panic because of the check immediately above.
        let job = job.unwrap();
        let chunck = job.chunck;
        let covs = job.covs;
        let metric = job.metric;
        let prefix = job.prefix;
        for file in chunck {
            let path = Path::new(&file);
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let arr = match covs.get(&file) {
                Some(arr) => arr.to_vec(),
                None => {
                    files_ignored.push(file);
                    continue;
                }
            };
            let (covered_lines, tot_lines) = get_covered_lines(&arr)?;
            all_cov_lines += covered_lines;
            all_tot_lines += tot_lines;
            let root = get_root(path)?;
            let sifis_plain = sifis_plain(&root, &arr, metric, false)?;
            let sifis_quantized = sifis_quantized(&root, &arr, metric, false)?;
            let crap = crap(&root, &arr, metric, None)?;
            let skunk = skunk_nosmells(&root, &arr, metric, None)?;
            let file_path = file.clone().split_off(prefix);
            let is_complex = check_complexity(sifis_plain, sifis_quantized, crap, skunk);
            let coverage = get_coverage_perc(&arr)? * 100.;
            res.push(Metrics {
                sifis_plain,
                sifis_quantized,
                crap,
                skunk,
                file: file_name,
                file_path,
                is_complex,
                coverage: f64::round(coverage * 100.0) / 100.0,
            });
        }
    }
    Ok((res, files_ignored, all_cov_lines, all_tot_lines))
}

fn chunck_vector(vec: Vec<String>, n_threads: usize) -> Vec<Vec<String>> {
    let chuncks = vec.chunks((vec.len() / n_threads).max(1));
    let mut result = Vec::<Vec<String>>::new();
    for c in chuncks {
        let mut v = Vec::<String>::new();
        for s in c {
            v.push(s.to_string());
        }
        result.push(v)
    }
    result
}

/// Concurrent version of get_metrics
pub fn get_metrics_concurrent<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: COMPLEXITY,
    n_threads: usize,
) -> Result<Output, SifisError> {
    let vec = match read_files(files_path.as_ref()) {
        Ok(vec) => vec,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_ref().display().to_string(),
            ))
        }
    };
    let file = match fs::read_to_string(json_path) {
        Ok(file) => file,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                json_path.as_ref().display().to_string(),
            ))
        }
    };
    let covs = read_json(file, files_path.as_ref().to_str().unwrap())?;
    let mut handlers = vec![];
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    let mut covered_lines = 0.;
    let mut tot_lines = 0.;
    let (sender, receiver) = unbounded();
    let chuncks = chunck_vector(vec, n_threads);
    for _ in 0..n_threads {
        let r = receiver.clone();
        let h = thread::spawn(
            move || -> Result<(Vec<Metrics>, Vec<String>, f64, f64), SifisError> { consumer(r) },
        );
        handlers.push(h);
    }
    let prefix = files_path.as_ref().to_str().unwrap().to_string().len();
    for chunck in chuncks {
        let job = JobItem {
            chunck: chunck.clone(),
            covs: covs.clone(),
            metric,
            prefix,
        };
        if let Err(_e) = sender.send(Some(job)) {
            return Err(SifisError::ConcurrentError());
        }
    }
    // stops all jobs
    for _ in 0..n_threads {
        if let Err(_e) = sender.send(None) {
            return Err(SifisError::ConcurrentError());
        }
    }
    for handle in handlers {
        let result = match handle.join().unwrap() {
            Ok(res) => res,
            Err(_err) => return Err(SifisError::ConcurrentError()),
        };
        for r in result.0 {
            res.push(r);
        }
        for f in result.1 {
            files_ignored.push(f);
        }
        covered_lines += result.2;
        tot_lines += result.3;
    }
    files_ignored.sort();
    res.sort_by(|a, b| a.file.cmp(&b.file));
    let (avg, max, min, complex_files) = get_cumulative_values(&res);
    res.push(avg);
    res.push(max);
    res.push(min);
    let project_coverage = covered_lines / tot_lines * 100.0;
    Ok((
        res,
        files_ignored,
        complex_files,
        f64::round(project_coverage * 100.) / 100.,
    ))
}

struct JobItemCovDir {
    chunck: Vec<String>,
    covs: HashMap<String, Covdir>,
    metric: COMPLEXITY,
    prefix: usize,
}

type JobReceiverCovDir = Receiver<Option<JobItemCovDir>>;
fn consumer_covdir(receiver: JobReceiverCovDir) -> Result<(Vec<Metrics>, Vec<String>), SifisError> {
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    while let Ok(job) = receiver.recv() {
        if job.is_none() {
            break;
        }
        // Cannot panic because of the check immediately above.
        let job = job.unwrap();
        let chunck = job.chunck;
        let covs = job.covs;
        let metric = job.metric;
        let prefix = job.prefix;
        for file in chunck {
            let path = Path::new(&file);
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let covdir = match covs.get(&file) {
                Some(covdir) => covdir,
                None => {
                    files_ignored.push(file);
                    continue;
                }
            };
            let arr = covdir.arr.clone();
            let coverage = Some(covdir.coverage);
            let root = get_root(path)?;
            let sifis_plain = sifis_plain(&root, &arr, metric, true)?;
            let sifis_quantized = sifis_quantized(&root, &arr, metric, true)?;
            let crap = crap(&root, &arr, metric, coverage)?;
            let skunk = skunk_nosmells(&root, &arr, metric, coverage)?;
            let file_path = file.clone().split_off(prefix);
            let is_complex = check_complexity(sifis_plain, sifis_quantized, crap, skunk);
            res.push(Metrics {
                sifis_plain,
                sifis_quantized,
                crap,
                skunk,
                file: file_name,
                file_path,
                is_complex,
                coverage: covdir.coverage,
            });
        }
    }
    Ok((res, files_ignored))
}
/// Concurrent version of get_metrics usign covdir format
pub fn get_metrics_concurrent_covdir<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: COMPLEXITY,
    n_threads: usize,
) -> Result<Output, SifisError> {
    let vec = match read_files(files_path.as_ref()) {
        Ok(vec) => vec,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_ref().display().to_string(),
            ))
        }
    };
    let file = match fs::read_to_string(json_path) {
        Ok(file) => file,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                json_path.as_ref().display().to_string(),
            ))
        }
    };
    let covs = read_json_covdir(file, files_path.as_ref().to_str().unwrap())?;
    let mut handlers = vec![];
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    let (sender, receiver) = unbounded();
    let chuncks = chunck_vector(vec, n_threads);
    for _ in 0..n_threads {
        let r = receiver.clone();
        let h = thread::spawn(move || -> Result<(Vec<Metrics>, Vec<String>), SifisError> {
            consumer_covdir(r)
        });
        handlers.push(h);
    }
    let prefix = files_path.as_ref().to_str().unwrap().to_string().len();
    for chunck in chuncks {
        let job = JobItemCovDir {
            chunck: chunck.clone(),
            covs: covs.clone(),
            metric,
            prefix,
        };
        if let Err(_e) = sender.send(Some(job)) {
            return Err(SifisError::ConcurrentError());
        }
    }
    // stops all jobs
    for _ in 0..n_threads {
        if let Err(_e) = sender.send(None) {
            return Err(SifisError::ConcurrentError());
        }
    }
    for handle in handlers {
        let result = match handle.join().unwrap() {
            Ok(res) => res,
            Err(_err) => return Err(SifisError::ConcurrentError()),
        };
        for r in result.0 {
            res.push(r);
        }
        for f in result.1 {
            files_ignored.push(f);
        }
    }
    files_ignored.sort();
    res.sort_by(|a, b| a.file.cmp(&b.file));
    let (avg, max, min, complex_files) = get_cumulative_values(&res);
    res.push(avg);
    res.push(max);
    res.push(min);
    let project_coverage = covs.get(&("PROJECT_ROOT".to_string())).unwrap().coverage;
    Ok((res, files_ignored, complex_files, project_coverage))
}
///Prints the reulst of the get_metric function in a csv file
/// the structure is the following :
/// FILE,SIFIS PLAIN,SIFIS QUANTAZED,CRAP,SKUNK
pub fn print_metrics_to_csv<A: AsRef<Path> + Copy>(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    csv_path: A,
    project_coverage: f64,
) -> Result<(), SifisError> {
    export_to_csv(
        csv_path.as_ref(),
        metrics,
        files_ignored,
        complex_files,
        project_coverage,
    )
}

pub fn print_metrics_to_json<A: AsRef<Path> + Copy>(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    json_output: A,
    project_folder: A,
    project_coverage: f64,
) -> Result<(), SifisError> {
    export_to_json(
        project_folder.as_ref(),
        json_output.as_ref(),
        metrics,
        files_ignored,
        complex_files,
        project_coverage,
    )
}
