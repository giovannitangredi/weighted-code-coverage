# SIFIS RUST

This repository contains a Rust language implementation of the Sifis-Home algoritm that  relate code coverage and code complexity explaned by Luca Ardito and others for Sifis-Home.


For more info check : https://www.sifis-home.eu/wp-content/uploads/2021/10/D2.2_SIFIS-Home_v1.0-to-submit.pdf (section 2.4.1).

## Usage

Run the project with the following command:
```
cargo run *files_path* *json_path* 
```

files_path : The relative path for the folder with the files to analyze. If it is a folder add a "/" at the end.

json_path : the path to the json coveralls file obtain from grcov.

Exemple : 
```
cargo run ./path/to/files/ path/to/json/json_file.json
```


## License
Distributed under the terms of the MIT license -
See LICENSE for details.