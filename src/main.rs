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
}

fn main() -> Result<(), SifisError> {
    let args = Args::parse();
    let metric_to_use = if args.cognitive {
        COMPLEXITY::COGNITIVE
    } else {
        COMPLEXITY::CYCLOMATIC
    };
    let (metrics, files_ignored) = get_metrics(&args.path_file, &args.path_json, metric_to_use)?;
    match &args.path_csv {
        Some(csv) => print_metrics_to_csv(metrics.clone(), files_ignored.clone(), csv)?,
        None => (),
    };
    match &args.json_output {
        Some(json) => print_metrics_to_json(
            metrics.clone(),
            files_ignored.clone(),
            json,
            &args.path_file,
        )?,
        None => (),
    };
    get_metrics_output(metrics, files_ignored)
}
