use clap::Parser;
use std::path::PathBuf;
use wcc::utility::SifisError;
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
    /// Path to the grcov json in coveralls format
    #[clap(
        long = "csv",
        parse(from_os_str),
        value_name = "CSV"
    )]
    path_csv: Option<PathBuf>,
}

fn main() -> Result<(), SifisError> {
    let args = Args::parse();
    match &args.path_csv {
        Some(csv) => print_metrics_to_csv(&args.path_file, &args.path_json,csv)?,
        None => (),
    };
    get_metrics_output(&args.path_file, &args.path_json)
}
