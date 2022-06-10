pub mod error;
pub mod metrics;
pub mod output;
pub mod utility;
use crate::error::Error;

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::*;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam::channel::{unbounded, Receiver};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use crate::metrics::crap::crap;
use crate::metrics::sifis::{sifis_plain, sifis_quantized};
use crate::metrics::skunk::skunk_nosmells;
use crate::output::*;
use crate::utility::*;

#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub struct MetricsConfig {
    sifis_plain: f64,
    sifis_quantized: f64,
    crap: f64,
    skunk: f64,
    file: String,
    file_path: String,
    is_complex: bool,
    coverage: f64,
}
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
impl Metrics {
    pub fn new(mc: MetricsConfig) -> Self {
        Self {
            sifis_plain: mc.sifis_plain,
            sifis_quantized: mc.sifis_quantized,
            crap: mc.crap,
            skunk: mc.skunk,
            file: mc.file,
            file_path: mc.file_path,
            is_complex: mc.is_complex,
            coverage: mc.coverage,
        }
    }
    pub fn avg() -> Self {
        Self {
            sifis_plain: 0.,
            sifis_quantized: 0.,
            crap: 0.,
            skunk: 0.,
            file: "AVG".to_string(),
            file_path: "-".to_string(),
            is_complex: false,
            coverage: 0.,
        }
    }
    pub fn min() -> Self {
        Self {
            sifis_plain: 0.,
            sifis_quantized: 0.,
            crap: 0.,
            skunk: 0.,
            file: "MIN".to_string(),
            file_path: "-".to_string(),
            is_complex: false,
            coverage: 0.,
        }
    }
    pub fn max() -> Self {
        Self {
            sifis_plain: 0.,
            sifis_quantized: 0.,
            crap: 0.,
            skunk: 0.,
            file: "MAX".to_string(),
            file_path: "-".to_string(),
            is_complex: false,
            coverage: 0.,
        }
    }
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
) -> Result<(), Error> {
    println!(
        "{0: <20} | {1: <20} | {2: <20} | {3: <20} | {4: <20} | {5: <20} | {6: <30}",
        "FILE", "SIFIS PLAIN", "SIFIS QUANTIZED", "CRAP", "SKUNKSCORE", "IS_COMPLEX", "FILE PATH"
    );
    metrics.iter().for_each(|m| {
        println!(
            "{0: <20} | {1: <20.3} | {2: <20.3} | {3: <20.3} | {4: <20.3} | {5: <20} | {6: <30}",
            m.file, m.sifis_plain, m.sifis_quantized, m.crap, m.skunk, m.is_complex, m.file_path
        );
    });
    println!("FILES IGNORED: {}", files_ignored.len());
    println!("COMPLEX FILES: {}", complex_files.len());
    Ok(())
}

/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// if the a file is not found in the json that files will be skipped
/// It returns the  tuple (res, files_ignored, complex_files, project_coverage)
pub fn get_metrics<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: Complexity,
    thresholds: &[f64],
) -> Result<Output, Error> {
    if thresholds.len() != 4 {
        return Err(Error::ThresholdsError());
    }
    let vec = read_files(files_path.as_ref())?;
    let mut covered_lines = 0.;
    let mut tot_lines = 0.;
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    let file = fs::read_to_string(json_path)?;
    let covs = read_json(
        file,
        files_path
            .as_ref()
            .to_str()
            .ok_or(Error::PathConversionError())?,
    )?;
    for path in vec {
        let p = Path::new(&path);
        let file = p
            .file_name()
            .ok_or(Error::PathConversionError())?
            .to_str()
            .ok_or(Error::PathConversionError())?
            .to_string();
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
        let (sifis_plain, _sum) = sifis_plain(&root, &arr, metric, false)?;
        let (sifis_quantized, _sum) = sifis_quantized(&root, &arr, metric, false)?;
        let crap = crap(&root, &arr, metric, None)?;
        let skunk = skunk_nosmells(&root, &arr, metric, None)?;
        let file_path = path.clone().split_off(
            files_path
                .as_ref()
                .to_str()
                .ok_or(Error::PathConversionError())?
                .len(),
        );
        let is_complex = check_complexity(sifis_plain, sifis_quantized, crap, skunk, thresholds);
        let coverage = get_coverage_perc(&arr)? * 100.;

        res.push(Metrics::new(MetricsConfig {
            sifis_plain,
            sifis_quantized,
            crap,
            skunk,
            file,
            file_path,
            is_complex,
            coverage: f64::round(coverage * 100.0) / 100.0,
        }));
    }
    let (avg, max, min, complex_files) = get_cumulative_values(&res);
    res.push(avg);
    res.push(max);
    res.push(min);
    let project_coverage = covered_lines / tot_lines;
    Ok((res, files_ignored, complex_files, project_coverage))
}

// job received by the consumer threads
#[derive(Clone)]
struct JobItem {
    chunk: Vec<String>,
    covs: HashMap<String, Vec<Value>>,
    metric: Complexity,
    prefix: usize,
    thresholds: Vec<f64>,
}
impl JobItem {
    fn new(
        chunk: Vec<String>,
        covs: HashMap<String, Vec<Value>>,
        metric: Complexity,
        prefix: usize,
        thresholds: Vec<f64>,
    ) -> Self {
        Self {
            chunk,
            covs,
            metric,
            prefix,
            thresholds,
        }
    }
}

impl fmt::Debug for JobItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Job: chunks:{:?}, metric:{}, prefix:{:?}, thresholds: {:?}",
            self.chunk, self.metric, self.prefix, self.thresholds
        )
    }
}

// Configuration shared by all threads with all the data that must be returned
#[derive(Clone, Default, Debug)]
pub struct Config {
    res: Arc<Mutex<Vec<Metrics>>>,
    files_ignored: Arc<Mutex<Vec<String>>>,
    covered_lines: Arc<Mutex<f64>>,
    total_lines: Arc<Mutex<f64>>,
    sifis_plain_sum: Arc<Mutex<f64>>,
    sifis_quantized_sum: Arc<Mutex<f64>>,
    ploc_sum: Arc<Mutex<f64>>,
    comp_sum: Arc<Mutex<f64>>,
}

impl Config {
    fn new() -> Self {
        Self {
            res: Arc::new(Mutex::new(Vec::<Metrics>::new())),
            files_ignored: Arc::new(Mutex::new(Vec::<String>::new())),
            sifis_plain_sum: Arc::new(Mutex::new(0.)),
            sifis_quantized_sum: Arc::new(Mutex::new(0.)),
            ploc_sum: Arc::new(Mutex::new(0.)),
            comp_sum: Arc::new(Mutex::new(0.)),
            covered_lines: Arc::new(Mutex::new(0.)),
            total_lines: Arc::new(Mutex::new(0.)),
        }
    }
    fn clone(&self) -> Self {
        Self {
            res: Arc::clone(&self.res),
            files_ignored: Arc::clone(&self.files_ignored),
            sifis_plain_sum: Arc::clone(&self.sifis_plain_sum),
            sifis_quantized_sum: Arc::clone(&self.sifis_quantized_sum),
            ploc_sum: Arc::clone(&self.ploc_sum),
            comp_sum: Arc::clone(&self.comp_sum),
            covered_lines: Arc::clone(&self.covered_lines),
            total_lines: Arc::clone(&self.total_lines),
        }
    }
}

type JobReceiver = Receiver<Option<JobItem>>;

// Consumer function run by ead independent thread
fn consumer(receiver: JobReceiver, cfg: &Config) -> Result<(), Error> {
    // Get all shared data
    let files_ignored = &cfg.files_ignored;
    let res = &cfg.res;
    let all_cov_lines = &cfg.covered_lines;
    let all_tot_lines = &cfg.total_lines;
    let sifis_plain_sum = &cfg.sifis_plain_sum;
    let sifis_quantized_sum = &cfg.sifis_quantized_sum;
    let ploc_sum = &cfg.ploc_sum;
    let comp_sum = &cfg.comp_sum;
    while let Ok(job) = receiver.recv() {
        if job.is_none() {
            break;
        }
        // Cannot panic because of the check immediately above.
        let job = job.unwrap();
        let chunk = job.chunk;
        let covs = job.covs;
        let metric = job.metric;
        let prefix = job.prefix;
        let thresholds = job.thresholds;
        // For each file in the chunk received
        for file in chunk {
            let path = Path::new(&file);
            let file_name = path
                .file_name()
                .ok_or(Error::PathConversionError())?
                .to_str()
                .ok_or(Error::PathConversionError())?
                .to_string();
            // Get the coverage vector from the coveralls file
            // if not present the file will be added to the files ignored
            let arr = match covs.get(&file) {
                Some(arr) => arr.to_vec(),
                None => {
                    let mut f = files_ignored.lock()?;
                    f.push(file);
                    continue;
                }
            };
            let (covered_lines, tot_lines) = get_covered_lines(&arr)?;
            debug!(
                "File: {:?} covered lines: {}  total lines: {}",
                file, covered_lines, tot_lines
            );
            let root = get_root(path)?;
            let ploc = root.metrics.loc.ploc();
            let comp = match metric {
                Complexity::Cyclomatic => root.metrics.cyclomatic.cyclomatic_sum(),
                Complexity::Cognitive => root.metrics.cognitive.cognitive_sum(),
            };
            let (sifis_plain, sp_sum) = sifis_plain(&root, &arr, metric, false)?;
            let (sifis_quantized, sq_sum) = sifis_quantized(&root, &arr, metric, false)?;
            let crap = crap(&root, &arr, metric, None)?;
            let skunk = skunk_nosmells(&root, &arr, metric, None)?;
            let file_path = file.clone().split_off(prefix);
            let is_complex =
                check_complexity(sifis_plain, sifis_quantized, crap, skunk, &thresholds);
            let coverage = get_coverage_perc(&arr)? * 100.;
            // Upgrade all the global variables and add metrics to the result and complex_files
            let mut res = res.lock()?;
            let mut all_cov_lines = all_cov_lines.lock()?;
            let mut all_tot_lines = all_tot_lines.lock()?;
            let mut sifis_plain_sum = sifis_plain_sum.lock()?;
            let mut sifis_quantized_sum = sifis_quantized_sum.lock()?;
            let mut ploc_sum = ploc_sum.lock()?;
            let mut comp_sum = comp_sum.lock()?;
            *all_cov_lines += covered_lines;
            *all_tot_lines += tot_lines;
            *ploc_sum += ploc;
            *sifis_plain_sum += sp_sum;
            *sifis_quantized_sum += sq_sum;
            *comp_sum += comp;
            res.push(Metrics::new(MetricsConfig {
                sifis_plain,
                sifis_quantized,
                crap,
                skunk,
                file: file_name,
                file_path,
                is_complex,
                coverage: f64::round(coverage * 100.0) / 100.0,
            }));
        }
    }
    Ok(())
}
// Chunks the vector of files in multiple chunk to be used by threads
// It will return number of chunk with the same number of elements usually equal
// Or very close to n_threads
fn chunk_vector(vec: Vec<String>, n_threads: usize) -> Vec<Vec<String>> {
    let chunks = vec.chunks((vec.len() / n_threads).max(1));
    chunks
        .map(|chunk| chunk.iter().map(|c| c.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>()
}

/// This Function get the folder of the repo to analyzed and the path to the coveralls file obtained using grcov
/// It also takes as arguments the complexity metrics that must be used between cognitive or cyclomatic
/// If the a file is not found in the json that files will be skipped
/// It returns the  tuple (res, files_ignored, complex_files, project_coverage)
pub fn get_metrics_concurrent<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: Complexity,
    n_threads: usize,
    thresholds: &[f64],
) -> Result<Output, Error> {
    if thresholds.len() != 4 {
        return Err(Error::ThresholdsError());
    }
    // Take all the files starting from the given project folder
    let vec = read_files(files_path.as_ref())?;
    // Read coveralls file to string and then get all the coverage vectors
    let file = fs::read_to_string(json_path)?;
    let covs = read_json(
        file,
        files_path
            .as_ref()
            .to_str()
            .ok_or(Error::PathConversionError())?,
    )?;
    let mut handlers = vec![];
    // Create a new vonfig with  all needed mutexes
    let cfg = Config::new();
    let (sender, receiver) = unbounded();
    // Chunks the files vector
    let chunks = chunk_vector(vec, n_threads);
    debug!("Files divided in {} chunks", chunks.len());
    debug!("Launching all {} threads", n_threads);
    for _ in 0..n_threads {
        let r = receiver.clone();
        let config = cfg.clone();
        // Launch n_threads consume threads
        let h = thread::spawn(move || -> Result<(), Error> { consumer(r, &config) });
        handlers.push(h);
    }
    let prefix = files_path
        .as_ref()
        .to_str()
        .ok_or(Error::PathConversionError())?
        .to_string()
        .len();
    // Send all chunks to the consumers
    chunks.iter().try_for_each(|chunk: &Vec<String>| {
        let job = JobItem::new(
            chunk.to_vec(),
            covs.clone(),
            metric,
            prefix,
            thresholds.to_vec(),
        );
        debug!("Sending job: {:?}", job);
        if let Err(_e) = sender.send(Some(job)) {
            return Err(Error::ConcurrentError());
        }
        Ok(())
    })?;
    // Stops all consumers by poisoning them
    debug!("Poisoning Threads...");
    handlers.iter().try_for_each(|_| {
        if let Err(_e) = sender.send(None) {
            return Err(Error::ConcurrentError());
        }
        Ok(())
    })?;
    // Wait the all consumers are  finished
    debug!("Waiting threads to finish...");
    for handle in handlers {
        handle.join()??;
    }
    let mut files_ignored = cfg.files_ignored.lock()?;
    let mut res = cfg.res.lock()?;
    let covered_lines = cfg.covered_lines.lock()?;
    let tot_lines = cfg.total_lines.lock()?;
    let project_coverage = *covered_lines / *tot_lines * 100.0;
    let project_metric = get_project_metrics(project_coverage, &cfg)?;
    files_ignored.sort();
    res.sort_by(|a, b| a.file.cmp(&b.file));
    // Get AVG MIN MAX and complex files
    let (avg, max, min, complex_files) = get_cumulative_values(&res);
    res.push(project_metric);
    res.push(avg);
    res.push(max);
    res.push(min);
    Ok((
        (*res).clone(),
        (*files_ignored).clone(),
        complex_files,
        f64::round(project_coverage * 100.) / 100.,
    ))
}

// Job received by the consumer threads for the covdir version
struct JobItemCovDir {
    chunk: Vec<String>,
    covs: HashMap<String, Covdir>,
    metric: Complexity,
    prefix: usize,
    thresholds: Vec<f64>,
}

impl JobItemCovDir {
    fn new(
        chunk: Vec<String>,
        covs: HashMap<String, Covdir>,
        metric: Complexity,
        prefix: usize,
        thresholds: Vec<f64>,
    ) -> Self {
        Self {
            chunk,
            covs,
            metric,
            prefix,
            thresholds,
        }
    }
}
impl fmt::Debug for JobItemCovDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Job: chunks:{:?}, metric:{}, prefix:{:?}, thresholds: {:?}",
            self.chunk, self.metric, self.prefix, self.thresholds
        )
    }
}

type JobReceiverCovDir = Receiver<Option<JobItemCovDir>>;

// Consumer thread for the covdir format
fn consumer_covdir(receiver: JobReceiverCovDir, cfg: &Config) -> Result<(), Error> {
    // Get all shared variables
    let files_ignored = &cfg.files_ignored;
    let res = &cfg.res;
    let sifis_plain_sum = &cfg.sifis_plain_sum;
    let sifis_quantized_sum = &cfg.sifis_quantized_sum;
    let ploc_sum = &cfg.ploc_sum;
    let comp_sum = &cfg.comp_sum;
    while let Ok(job) = receiver.recv() {
        if job.is_none() {
            break;
        }
        // Cannot panic because of the check immediately above.
        let job = job.unwrap();
        let chunk = job.chunk;
        let covs = job.covs;
        let metric = job.metric;
        let prefix = job.prefix;
        let thresholds = job.thresholds;
        // For each file in the chunk
        for file in chunk {
            let path = Path::new(&file);
            let file_name = path
                .file_name()
                .ok_or(Error::PathConversionError())?
                .to_str()
                .ok_or(Error::PathConversionError())?
                .to_string();
            // Get the coverage vector from the coveralls file
            // If not present the file will be added to the files ignored
            let covdir = match covs.get(&file) {
                Some(covdir) => covdir,
                None => {
                    let mut f = files_ignored.lock()?;
                    f.push(file);
                    continue;
                }
            };
            let arr = &covdir.arr;
            let coverage = Some(covdir.coverage);
            let root = get_root(path)?;
            let ploc = root.metrics.loc.ploc();
            let comp = match metric {
                Complexity::Cyclomatic => root.metrics.cyclomatic.cyclomatic_sum(),
                Complexity::Cognitive => root.metrics.cognitive.cognitive_sum(),
            };
            let (sifis_plain, sp_sum) = sifis_plain(&root, arr, metric, true)?;
            let (sifis_quantized, sq_sum) = sifis_quantized(&root, arr, metric, true)?;
            let crap = crap(&root, arr, metric, coverage)?;
            let skunk = skunk_nosmells(&root, arr, metric, coverage)?;
            let file_path = file.clone().split_off(prefix);
            let is_complex =
                check_complexity(sifis_plain, sifis_quantized, crap, skunk, &thresholds);
            let mut res = res.lock()?;
            let mut sifis_plain_sum = sifis_plain_sum.lock()?;
            let mut sifis_quantized_sum = sifis_quantized_sum.lock()?;
            let mut ploc_sum = ploc_sum.lock()?;
            let mut comp_sum = comp_sum.lock()?;
            // Update all shared variables
            *ploc_sum += ploc;
            *sifis_plain_sum += sp_sum;
            *sifis_quantized_sum += sq_sum;
            *comp_sum += comp;
            let coverage = covdir.coverage;
            res.push(Metrics::new(MetricsConfig {
                sifis_plain,
                sifis_quantized,
                crap,
                skunk,
                file: file_name,
                file_path,
                is_complex,
                coverage: f64::round(coverage * 100.0) / 100.0,
            }));
        }
    }
    Ok(())
}

/// This Function get the folder of the repo to analyzed and the path to the covdir file obtained using grcov
/// It also takes as arguments the complexity metrics that must be used between cognitive or cyclomatic
/// If the a file is not found in the json that files will be skipped
/// It returns the  tuple (res, files_ignored, complex_files, project_coverage)
pub fn get_metrics_concurrent_covdir<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: Complexity,
    n_threads: usize,
    thresholds: &[f64],
) -> Result<Output, Error> {
    if thresholds.len() != 4 {
        return Err(Error::ThresholdsError());
    }
    // Get all the files from project folder
    let vec = read_files(files_path.as_ref())?;
    // Read covdir json and obtain all coverage information
    let file = fs::read_to_string(json_path)?;
    let covs = read_json_covdir(
        file,
        files_path
            .as_ref()
            .to_str()
            .ok_or(Error::PathConversionError())?,
    )?;
    let mut handlers = vec![];
    // Create a new Config all needed mutexes
    let cfg = Config::new();
    let (sender, receiver) = unbounded();
    // Chunks the files vector
    let chunks = chunk_vector(vec, n_threads);
    debug!("Files divided in {} chunks", chunks.len());
    debug!("Launching all {} threads", n_threads);
    // Launch n_threads consumer threads
    for _ in 0..n_threads {
        let r = receiver.clone();
        let config = cfg.clone();
        let h = thread::spawn(move || -> Result<(), Error> { consumer_covdir(r, &config) });
        handlers.push(h);
    }
    let prefix = files_path
        .as_ref()
        .to_str()
        .ok_or(Error::PathConversionError())?
        .to_string()
        .len();
    chunks.iter().try_for_each(|chunk| {
        let job = JobItemCovDir::new(
            chunk.to_vec(),
            covs.clone(),
            metric,
            prefix,
            thresholds.to_vec(),
        );
        debug!("Sending job: {:?}", job);
        if let Err(_e) = sender.send(Some(job)) {
            return Err(Error::ConcurrentError());
        }
        Ok(())
    })?;
    debug!("Poisoning threads...");
    // Stops all jobs by poisoning
    handlers.iter().try_for_each(|_| {
        if let Err(_e) = sender.send(None) {
            return Err(Error::ConcurrentError());
        }
        Ok(())
    })?;
    debug!("Waiting for threads to finish...");
    // Wait the termination of all consumers
    for handle in handlers {
        handle.join()??;
    }
    let mut files_ignored = cfg.files_ignored.lock()?;
    let mut res = cfg.res.lock()?;
    let project_coverage = covs
        .get(&("PROJECT_ROOT".to_string()))
        .ok_or(Error::HashMapError())?
        .coverage;
    // Get final  metrics for all the project
    let project_metric = get_project_metrics(project_coverage, &cfg)?;
    files_ignored.sort();
    res.sort_by(|a, b| a.file.cmp(&b.file));
    // Get AVG MIN MAX and complex files
    let (avg, max, min, complex_files) = get_cumulative_values(&res);
    res.push(project_metric);
    res.push(avg);
    res.push(max);
    res.push(min);
    Ok((
        (*res).clone(),
        (*files_ignored).clone(),
        complex_files,
        project_coverage,
    ))
}

/// Prints the the given  metrics ,files ignored and complex files  in a csv format
/// The structure is the following :
/// "FILE","SIFIS PLAIN","SIFIS QUANTIZED","CRAP","SKUNK","IGNORED","IS COMPLEX","FILE PATH",
pub fn print_metrics_to_csv<A: AsRef<Path> + Copy>(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    csv_path: A,
    project_coverage: f64,
) -> Result<(), Error> {
    debug!("Exporting to csv...");
    export_to_csv(
        csv_path.as_ref(),
        metrics,
        files_ignored,
        complex_files,
        project_coverage,
    )
}

/// Prints the the given  metrics ,files ignored and complex files  in a json format
pub fn print_metrics_to_json<A: AsRef<Path> + Copy>(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    json_output: A,
    project_folder: A,
    project_coverage: f64,
) -> Result<(), Error> {
    debug!("Exporting to json...");
    export_to_json(
        project_folder.as_ref(),
        json_output.as_ref(),
        metrics,
        files_ignored,
        complex_files,
        project_coverage,
    )
}

#[cfg(test)]
mod tests {

    use super::*;
    const JSON: &str = "./data/seahorse/seahorse.json";
    const COVDIR: &str = "./data/seahorse/covdir.json";
    const PROJECT: &str = "./data/seahorse/";
    const IGNORED: &str = "./data/seahorse/src/action.rs";

    #[inline(always)]
    fn compare_float(a: f64, b: f64) -> bool {
        a - b < 1.0e-5
    }

    #[test]
    fn test_metrics_coveralls_cyclomatic() {
        let json = Path::new(JSON);
        let project = Path::new(PROJECT);
        let ignored = Path::new(IGNORED);
        let (metrics, files_ignored, _, _) = get_metrics_concurrent(
            project,
            json,
            Complexity::Cyclomatic,
            8,
            &[30., 1.5, 35., 30.],
        )
        .unwrap();
        let error = &metrics[3];
        let ma = &metrics[7];
        let h = &metrics[5];
        let app = &metrics[0];
        let cont = &metrics[2];

        assert_eq!(files_ignored.len(), 1);
        assert!(files_ignored[0] == ignored.as_os_str().to_str().unwrap());
        assert!(compare_float(error.sifis_plain, 0.53125));
        assert!(compare_float(error.sifis_quantized, 0.03125));
        assert!(compare_float(error.crap, 257.9411765));
        assert!(compare_float(error.skunk, 64.));
        assert!(compare_float(ma.sifis_plain, 0.));
        assert!(compare_float(ma.sifis_quantized, 0.));
        assert!(compare_float(ma.crap, 552.));
        assert!(compare_float(ma.skunk, 92.));
        assert!(compare_float(h.sifis_plain, 1.5));
        assert!(compare_float(h.sifis_quantized, 0.5));
        assert!(compare_float(h.crap, 3.));
        assert!(compare_float(h.skunk, 0.12));
        assert!(compare_float(app.sifis_plain, 79.2147806));
        assert!(compare_float(app.sifis_quantized, 0.792147806));
        assert!(compare_float(app.crap, 123.974085565377));
        assert!(compare_float(app.skunk, 53.53535354));
        assert!(compare_float(cont.sifis_plain, 24.31578947));
        assert!(compare_float(cont.sifis_quantized, 0.7368421053));
        assert!(compare_float(cont.crap, 33.46814484));
        assert!(compare_float(cont.skunk, 9.962264151));
    }

    #[test]
    fn test_metrics_coveralls_cognitive() {
        let json = Path::new(JSON);
        let project = Path::new(PROJECT);
        let ignored = Path::new(IGNORED);
        let (metrics, files_ignored, _, _) = get_metrics_concurrent(
            project,
            json,
            Complexity::Cognitive,
            8,
            &[30., 1.5, 35., 30.],
        )
        .unwrap();
        let error = &metrics[3];
        let ma = &metrics[7];
        let h = &metrics[5];
        let app = &metrics[0];
        let cont = &metrics[2];

        assert_eq!(files_ignored.len(), 1);
        assert!(files_ignored[0] == ignored.as_os_str().to_str().unwrap());
        assert!(compare_float(error.sifis_plain, 0.53125));
        assert!(compare_float(error.sifis_quantized, 0.03125));
        assert!(compare_float(error.crap, 257.9411765));
        assert!(compare_float(error.skunk, 64.));
        assert!(compare_float(ma.sifis_plain, 0.));
        assert!(compare_float(ma.sifis_quantized, 0.));
        assert!(compare_float(ma.crap, 72.));
        assert!(compare_float(ma.skunk, 32.));
        assert!(compare_float(h.sifis_plain, 1.5));
        assert!(compare_float(h.sifis_quantized, 0.5));
        assert!(compare_float(h.crap, 3.));
        assert!(compare_float(h.skunk, 0.12));
        assert!(compare_float(app.sifis_plain, 66.5404157));
        assert!(compare_float(app.sifis_quantized, 0.792147806));
        assert!(compare_float(app.crap, 100.9161148));
        assert!(compare_float(app.skunk, 44.97));
        assert!(compare_float(cont.sifis_plain, 18.42105263));
        assert!(compare_float(cont.sifis_quantized, 0.88721804511));
        assert!(compare_float(cont.crap, 25.26867817));
        assert!(compare_float(cont.skunk, 7.547169811));
    }

    #[test]
    fn test_metrics_covdir_cyclomatic() {
        let covdir = Path::new(COVDIR);
        let project = Path::new(PROJECT);
        let ignored = Path::new(IGNORED);
        let (metrics, files_ignored, _, _) = get_metrics_concurrent_covdir(
            project,
            covdir,
            Complexity::Cyclomatic,
            8,
            &[30., 1.5, 35., 30.],
        )
        .unwrap();
        let error = &metrics[3];
        let ma = &metrics[7];
        let h = &metrics[5];
        let app = &metrics[0];
        let cont = &metrics[2];

        assert_eq!(files_ignored.len(), 1);
        assert!(files_ignored[0] == ignored.as_os_str().to_str().unwrap());
        assert!(compare_float(error.sifis_plain, 0.53125));
        assert!(compare_float(error.sifis_quantized, 0.03125));
        assert!(compare_float(error.crap, 257.9592475));
        assert!(compare_float(error.skunk, 64.0016));
        assert!(compare_float(ma.sifis_plain, 0.));
        assert!(compare_float(ma.sifis_quantized, 0.));
        assert!(compare_float(ma.crap, 552.));
        assert!(compare_float(ma.skunk, 92.));
        assert!(compare_float(h.sifis_plain, 1.5));
        assert!(compare_float(h.sifis_quantized, 0.5));
        assert!(compare_float(h.crap, 3.));
        assert!(compare_float(h.skunk, 0.12));
        assert!(compare_float(app.sifis_plain, 79.2147806));
        assert!(compare_float(app.sifis_quantized, 0.792147806));
        assert!(compare_float(app.crap, 123.974085565377));
        assert!(compare_float(app.skunk, 53.53535354));
        assert!(compare_float(cont.sifis_plain, 24.31578947));
        assert!(compare_float(cont.sifis_quantized, 0.7368421053));
        assert!(compare_float(cont.crap, 33.46867171));
        assert!(compare_float(cont.skunk, 9.96599999));
    }

    #[test]
    fn test_metrics_covdir_cognitive() {
        let covdir = Path::new(COVDIR);
        let project = Path::new(PROJECT);
        let ignored = Path::new(IGNORED);
        let (metrics, files_ignored, _, _) = get_metrics_concurrent_covdir(
            project,
            covdir,
            Complexity::Cognitive,
            8,
            &[30., 1.5, 35., 30.],
        )
        .unwrap();
        let error = &metrics[3];
        let ma = &metrics[7];
        let h = &metrics[5];
        let app = &metrics[0];
        let cont = &metrics[2];
        assert_eq!(files_ignored.len(), 1);
        assert!(files_ignored[0] == ignored.as_os_str().to_str().unwrap());
        assert!(compare_float(error.sifis_plain, 0.53125));
        assert!(compare_float(error.sifis_quantized, 0.03125));
        assert!(compare_float(error.crap, 257.9411765));
        assert!(compare_float(error.skunk, 64.));
        assert!(compare_float(ma.sifis_plain, 0.));
        assert!(compare_float(ma.sifis_quantized, 0.));
        assert!(compare_float(ma.crap, 72.));
        assert!(compare_float(ma.skunk, 32.));
        assert!(compare_float(h.sifis_plain, 1.5));
        assert!(compare_float(h.sifis_quantized, 0.5));
        assert!(compare_float(h.crap, 3.));
        assert!(compare_float(h.skunk, 0.12));
        assert!(compare_float(app.sifis_plain, 66.5404157));
        assert!(compare_float(app.sifis_quantized, 0.792147806));
        assert!(compare_float(app.crap, 100.9161148));
        assert!(compare_float(app.skunk, 44.97));
        assert!(compare_float(cont.sifis_plain, 18.42105263));
        assert!(compare_float(cont.sifis_quantized, 0.88721804511));
        assert!(compare_float(cont.crap, 25.2689805));
        assert!(compare_float(cont.skunk, 7.54999999999));
    }
}
