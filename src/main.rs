use std::path::PathBuf;
use std::fs;
use std::collections::*;
use rust_code_analysis::{dump_root, metrics, RustParser, ParserTrait,read_file};
use serde_json::*;
fn main() {
    let paths = fs::read_dir("../rust-data-structures-main/src").unwrap();
    let file = fs::read_to_string("./src/coveralls.json").unwrap();
    let val : Value  =  serde_json::from_str(file.as_str()).unwrap();
    let   vec=val["source_files"] .as_array().unwrap();
    let mut covs = HashMap::<String,Vec<Value>>::new();
    for x in vec {
        let mut name = "../rust-data-structures-main/".to_string();
        name= name+x["name"].as_str().unwrap();
        covs.insert(name.to_string(),x["coverage"].as_array().unwrap().to_vec());
    }

    for path in paths {
        let  p = path.unwrap().path();
        let data = read_file(&p).unwrap();
        let parser = RustParser::new(data, &p, None);
        let space = metrics(&parser, &p).unwrap();
        //dump_root(&space).unwrap();
        let ploc = space.metrics.loc.ploc();
        let comp = space.metrics.cyclomatic.cyclomatic_sum();
        let rows = covs.get(&p.to_str().unwrap().to_string()).unwrap();
        let mut sum=0.0;
        for  i in 0..rows.len() {
            if !rows.get(i).unwrap().is_null() && (rows.get(i).unwrap().as_u64().unwrap()>0)
            {
                sum+=comp;
            }
        }
        let sifis = sum/ploc;
        println!("For {:?} the SIFIS value is {:?}",p.file_name().unwrap(),sifis);
        
    }
    
}
