mod sifis;
use crate::sifis::{get_sifis, SifisError};
use std::env;

struct Config {
    path_file: String,
    path_json: String,
}

impl Config {
    fn new(args: &[String]) -> Config {
        if args.len() != 3 {
            println!("Correct format: \n cargo run *project_path* *json_path*");
            panic!("not enough arguments");
        }
        let path_file = args[1].clone();
        let path_json = args[2].clone();

        Config {
            path_file,
            path_json,
        }
    }
}

fn main() -> Result<(), SifisError> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);
    match get_sifis(&config.path_file, &config.path_json) {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}
