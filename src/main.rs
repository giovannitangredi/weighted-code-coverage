mod sifis;
use crate::sifis::{get_metrics, SifisError};
use clap::Parser;
use std::path::PathBuf;

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

    /// path to the grcov json
    #[clap(
        short = 'j',
        long = "path_json",
        parse(from_os_str),
        value_name = "FILE"
    )]
    path_json: PathBuf,
}

fn main() -> Result<(), SifisError> {
    let args = Args::parse();
    match get_metrics(&args.path_file, &args.path_json) {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}
