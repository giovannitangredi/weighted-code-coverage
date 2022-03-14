pub mod crap;
pub mod sifis;
pub mod skunk;
pub mod utility;

use crate::crap::crap;
use crate::sifis::{sifis_plain, sifis_quantized};
use crate::skunk::skunk_nosmells;
use crate::utility::*;
use std::fs;
use std::path::*;

/// Struct with all the metrics computed for a single file
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub struct Metrics {
    sifis_plain: f64,
    sifis_quantized: f64,
    crap: f64,
    skunk: f64,
    file: String,
}
/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// It prints all the SIFIS, CRAP and SkunkScore values for all the Rust files in the folders
/// the output will be print as follows:
/// FILE       | SIFIS PLAIN | SIFIS QUANTIZED | CRAP       | SKUNKSCORE
/// if the a file is not found in the json the output will be shown as NaN
pub fn get_metrics_output<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
) -> Result<(), SifisError> {
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
    println!(
        "{0: <20} | {1: <20} | {2: <20} | {3: <20} | {4: <20}",
        "FILE", "SIFIS PLAIN", "SIFIS QUANTIZED", "CRAP", "SKUNKSCORE"
    );
    for path in vec {
        let p = Path::new(&path);
        let arr = match covs.get(&path) {
            Some(arr) => arr.to_vec(),
            None => {
                println!(
                    "{0: <20} | {1: <20} | {2: <20} | {3: <20} | {4: <20}",
                    p.file_name().unwrap().to_str().unwrap(),f64::NAN, f64::NAN, f64::NAN, f64::NAN
                );
                continue;  
            },
        };
        let sifis = sifis_plain(p, &arr)?;
        let sifis_quantized = sifis_quantized(p, &arr)?;
        let crap = crap(p, &arr)?;
        let skunk = skunk_nosmells(p, &arr)?;
        println!(
            "{0: <20} | {1: <20.3} | {2: <20.3} | {3: <20.3} | {4: <20.3}",
            p.file_name().unwrap().to_str().unwrap(),
            sifis,
            sifis_quantized,
            crap,
            skunk
        );
    }
    Ok(())
}

pub fn get_metrics<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
) -> Result<Vec<Metrics>, SifisError> {
    let vec = match read_files(files_path.as_ref()) {
        Ok(vec) => vec,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_ref().display().to_string(),
            ))
        }
    };
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
        let arr = match covs.get(&path) {
            Some(arr) => arr.to_vec(),
            None => {
                res.push(Metrics {
                    sifis_plain : f64::NAN,
                    sifis_quantized: f64::NAN,
                    crap: f64::NAN,
                    skunk: f64::NAN,
                    file : p.file_name().unwrap().to_str().unwrap().to_string(),
                });
                continue;  
            },
        };
        let file = p.file_name().unwrap().to_str().unwrap().to_string();
        let sifis_plain = sifis_plain(p, &arr)?;
        let sifis_quantized = sifis_quantized(p, &arr)?;
        let crap = crap(p, &arr)?;
        let skunk = skunk_nosmells(p, &arr)?;
        res.push(Metrics {
            sifis_plain,
            sifis_quantized,
            crap,
            skunk,
            file,
        });
    }
    Ok(res)
}

pub fn print_metrics_to_csv<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy, C: AsRef<Path> + Copy>( 
    files_path: A,
    json_path: B,
    csv_path: C
) -> Result<(),SifisError> {
    let metrics = get_metrics(files_path,json_path)?;
    export_to_csv(csv_path.as_ref(),metrics)
}