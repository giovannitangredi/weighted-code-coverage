use clap::Parser;
use std::path::PathBuf;
use wcc::utility::SifisError;
use wcc::utility::COMPLEXITY;
use wcc::*;

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
    /// Use cognitive metric instead of cyclomatic
    #[clap(
        long = "csv",
        parse(from_os_str),
        value_name = "CSV"
    )]
    path_csv: Option<PathBuf>,
     /// Path to the grcov json in coveralls format
     #[clap(
        long = "cognitive",
        short ='c',
        parse(from_flag)
    )]
    cognitive: bool,
}

fn main() -> Result<(), SifisError> {
    let args = Args::parse();
    let metric_to_use = if args.cognitive {
        COMPLEXITY::COGNITIVE
    } else {
        COMPLEXITY::CYCLOMATIC
    };
    match &args.path_csv {
        Some(csv) => print_metrics_to_csv(&args.path_file, &args.path_json,csv,metric_to_use)?,
        None => (),
    };
    get_metrics_output(&args.path_file, &args.path_json,metric_to_use)
}
