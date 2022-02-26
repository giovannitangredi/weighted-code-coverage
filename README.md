# WCC

This repository contains a Rust language implementation of the Sifis-Home algorithm that relate code coverage and code complexity explained by Luca Ardito and others for Sifis-Home.

For more info check : https://www.sifis-home.eu/wp-content/uploads/2021/10/D2.2_SIFIS-Home_v1.0-to-submit.pdf (section 2.4.1).

This repository uses rust code analysis for analyzing the project folder. (https://github.com/mozilla/rust-code-analysis/) and grcov for the json file to use in the application (https://github.com/mozilla/grcov).

## Usage

Run the project with the following command:
```
cargo run -- -p *files_path* -j *json_path* 
```

files_path : The relative path to the folder with the files to analyze. If it is a folder, add a "/" at the end.

json_path : The relative path to the json coveralls file obtained from grcov.

Exemple : 
```
cargo run --  -p ../rust-data-structures-main/ -j ./data/coveralls.json
```


## License
Distributed under the terms of the MIT license -
See LICENSE for details.