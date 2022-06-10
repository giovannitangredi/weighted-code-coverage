# Weighted Code Coverage

This repository contains the implementations of some Weighted Code Coverage algorithms
for some of the languages supported by rust-code-analysis.

The implemented algorithms are:
- Sifis-Home mechanism by Luca Ardito and others (https://www.sifis-home.eu/wp-content/uploads/2021/10/D2.2_SIFIS-Home_v1.0-to-submit.pdf (section 2.4.1))
- CRAP by Alberto Savoia and Bob Evans(https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans. )
- SkunkScore by Ernesto Tagwerker (https://www.fastruby.io/blog/code-quality/intruducing-skunk-stink-score-calculator.html )

This repository uses [rust code analysis](https://github.com/mozilla/rust-code-analysis/)
to analyze a project folder and [grcov](https://github.com/mozilla/grcov)
to produce the coverage data used as `weighted-code-coverage` input.

## Usage

Run `weighted-code-coverage` on a project with the following command:

USAGE:
    weighted-code-coverage [OPTIONS] --path_file <PATH_FILE> --path_json <PATH_JSON>

OPTIONS:
    -c, --complexity <COMPLEXITY>      Choose complexity metric to use [default: cyclomatic]
                                       [possible values: cyclomatic, cognitive]
        --csv <PATH_CSV>               Path where to save the output of the csv file
    -f, --json-format <JSON_FORMAT>    Specify the type of format used between coveralls and covdir
                                       [default: coveralls] [possible values: covdir, coveralls]
    -h, --help                         Print help information
    -j, --path_json <PATH_JSON>        Path to the grcov json in coveralls/covdir format
        --json <JSON_OUTPUT>           Path where to save the output of the json file
    -n, --n_threads <N_THREADS>        Number of threads to use for concurrency [default: 2]
    -p, --path_file <PATH_FILE>        Path to the project folder
    -t, --thresholds <THRESHOLDS>      Set four  thresholds in this order: SIFIS PLAIN, SIFIS
                                       QUANTIZED, CRAP, SKUNK
                                       
                                           All the values must be floats
                                       
                                           All Thresholds has 0 as minimum value, thus no threshold
                                       at all.
                                       
                                           SIFIS PLAIN has a max threshold of COMP*SLOC/PLOC
                                       
                                           SIFIS QUANTIZED has a max threshold of 2*SLOC/PLOC
                                       
                                           CRAP has a max threshold of COMP^2 +COMP
                                       
                                           SKUNK has a max threshold of COMP/25
                                       [default: 35.0,1.5,35.0,30.0]
    -v, --verbose                      Output the generated paths as they are produced
    -V, --version                      Print version information

Example:

```
weighted-code-coverage  --path_file /path/to/source/code --path_json /path/to/coveralls.json -c cyclomatic --json /path/to/output.json -f coveralls -t 35.0,1.5,35.0,30.0
```

## Steps to install and run weighted-code-coverage

- grcov needs a rust nightly version in order to work, so switch to it with: ``rustup default nightly``
- Install grcov latest version using cargo ``cargo install grcov``
- After grcov has been installed, install `llvm-tools component`:

```
rustup component add llvm-tools-preview
```

- `RUSTFLAGS` and `LLVM_PROFILE_FILE` environment variables need to be set in this way

```
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="your_name-%p-%m.profraw"
```

- Then go to the folder of the repository you need to analyze and run all tests with ``cargo test``
- After each test has been passed, some `.profraw` files are generated. To print out the json file with all the coverage information inside, run the following command:

```
grcov . --binary-path ./target/debug/ -t coveralls -s . --token YOUR_COVERALLS_TOKEN > coveralls.json
```

- At the end, launch `weighted-code-coverage` with your desired options

## License

Distributed under the terms of the MIT license - See LICENSE for details.
