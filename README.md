# Weighted Code Coverage

This repository contains implementations for some Weighted Code Coverage algorithms fir the Rust language.
These algorithms are actually implemented:
- Sifis-Home mechanism by Luca Ardito and others (https://www.sifis-home.eu/wp-content/uploads/2021/10/D2.2_SIFIS-Home_v1.0-to-submit.pdf (section 2.4.1))
- CRAP by Alberto Savoia and Bob Evans(https://testing.googleblog.com/2011/02/this-code-is-crap.html#:~:text=CRAP%20is%20short%20for%20Change,partner%20in%20crime%20Bob%20Evans. )
- SkunkScore by Ernesto Tagwerker (https://www.fastruby.io/blog/code-quality/intruducing-skunk-stink-score-calculator.html )

This repository uses rust code analysis for analyzing the project folder. (https://github.com/mozilla/rust-code-analysis/) and grcov for the json file to use in the application (https://github.com/mozilla/grcov).

## Usage

Run the project with the following command:
```
wcc --path_file <FILE> --path_json <FILE>
```

files_path : The relative path to the folder with the files to analyze. If it is a folder, add a "/" at the end.

json_path : The relative path to the json coveralls file obtained from grcov.

Exemple : 
```
wcc  --path_file ../rust-data-structures-main/ --path_json ./data/coveralls.json
```


## License
Distributed under the terms of the MIT license -
See LICENSE for details.