# Weighted Code Coverage

This repository contains implementations for some Weighted Code Coverage algorithms for  all the languages supported by rust-code-analysis.

These algorithms are actually implemented:
- Sifis-Home mechanism by Luca Ardito and others (https://www.sifis-home.eu/wp-content/uploads/2021/10/D2.2_SIFIS-Home_v1.0-to-submit.pdf (section 2.4.1))
- CRAP by Alberto Savoia and Bob Evans(https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans. )
- SkunkScore by Ernesto Tagwerker (https://www.fastruby.io/blog/code-quality/intruducing-skunk-stink-score-calculator.html )

This repository uses rust code analysis to analyze the project folder. (https://github.com/mozilla/rust-code-analysis/) and `grcov` to produce the `json` file that are used in the application (https://github.com/mozilla/grcov).

## Usage

Run the project with the following command:

```
wcc --path_file <FILE> --path_json <FILE>
```

OPTIONS:

    -c, --cognitive             Use cognitive metric instead of cyclomatic
        --csv <CSV_OUTPUT>      Path where to save the output of the csv file
    -h, --help                  Print help information
    -j, --path_json <FILE>      Path to the grcov json in coveralls format
        --json <JSON_OUTPUT>    Path where to save the output of the json file
    -p, --path_file <FILE>      Path to the project folder
    -V, --version               Print version information

Exemple:

```
wcc  --path_file ../rust-data-structures-main/ --path_json ./data/coveralls.json -c --json ./output/file.json
```

## Steps to install and run wcc

- grcov needs a rust nightly version in order to work at its best, so remember to switch to this version before using grcov: ``rustup default nightly``
- Install grcov latest version using cargo ``cargo install grcov``
- After grcov has been installed, install llvm-tools component using rustup 

```
rustup component add llvm-tools-preview
```

- `RUSTFLAGS` and `LLVM_PROFILE_FILE` environment variables need to be set in this way

```
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="your_name-%p-%m.profraw"
```

- Then go to the folder of the repository you need to analyze and run all tests with ``cargo test``
- After each test has passed, some `.profraw` files are generated. To print out the json file with all the coverage information run the following command:

```
grcov . --binary-path ./target/debug/ -t coveralls -s . --token YOUR_COVERALLS_TOKEN > coveralls.json
```

- At the end, launch `wcc` with your desired options

## License

Distributed under the terms of the MIT license - See LICENSE for details.
