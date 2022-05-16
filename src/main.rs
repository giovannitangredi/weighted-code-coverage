use clap::Parser;
use std::path::PathBuf;
use weighted_code_coverage::utility::SifisError;
use weighted_code_coverage::utility::COMPLEXITY;
use weighted_code_coverage::*;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the project folder
    #[clap(
        short = 'p',
        long = "path_file",
        parse(from_os_str),
        value_name = "FILE"
    )]
    path_file: PathBuf,

    /// Path to the grcov json in coveralls format
    #[clap(
        short = 'j',
        long = "path_json",
        parse(from_os_str),
        value_name = "FILE"
    )]
    path_json: PathBuf,
    /// Path where to save the output of the csv file
    #[clap(long = "csv", parse(from_os_str), value_name = "CSV_OUTPUT")]
    path_csv: Option<PathBuf>,
    /// Path where to save the output of the json file
    #[clap(long = "json", parse(from_os_str), value_name = "JSON_OUTPUT")]
    json_output: Option<PathBuf>,
    /// Use cognitive metric instead of cyclomatic
    #[clap(long = "cognitive", short = 'c', parse(from_flag))]
    cognitive: bool,

    /// Number of threads to use for concurrency
    #[clap(long = "n_threads", short = 'n', default_value_t = 8)]
    n_threads: usize,
    /// Path where to save the output of the json file
    #[clap(long = "json_type", short ='t', value_name = "String",default_value_t = String::from("coveralls"))]
    json_type: String,
}

fn main() -> Result<(), SifisError> {
    let args = Args::parse();
    let metric_to_use = if args.cognitive {
        COMPLEXITY::COGNITIVE
    } else {
        COMPLEXITY::CYCLOMATIC
    };
    if args.n_threads == 0 {
        panic!("Number of threads must be greater than 0!")
    }

    let (metrics, files_ignored, complex_files, project_coverage) = if args.json_type == "covdir" {
        get_metrics_concurrent_covdir(
            &args.path_file,
            &args.path_json,
            metric_to_use,
            args.n_threads,
        )?
    } else if args.json_type == "coveralls" {
        get_metrics_concurrent(
            &args.path_file,
            &args.path_json,
            metric_to_use,
            args.n_threads,
        )?
    } else {
        panic!("Wrong json type! Only covdir or coveralls are supported");
    };
    match &args.path_csv {
        Some(csv) => print_metrics_to_csv(
            metrics.clone(),
            files_ignored.clone(),
            complex_files.clone(),
            csv,
            project_coverage,
        )?,
        None => (),
    };
    match &args.json_output {
        Some(json) => print_metrics_to_json(
            metrics.clone(),
            files_ignored.clone(),
            complex_files.clone(),
            json,
            &args.path_file,
            project_coverage,
        )?,
        None => (),
    };
    //get_metrics_output(metrics, files_ignored, complex_files)
    Ok(())
}
