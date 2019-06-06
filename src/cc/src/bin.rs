use localc_cc_lib::*;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Default)]
struct Config {
    verbose: bool,
    input_files: Vec<String>,
    output_file: Option<String>,
}

impl Config {
    fn output_file(&self) -> String {
        match self.output_file.as_ref() {
            Some(f) => f.clone(),
            _ => "out.lcc".to_string(),
        }
    }
}

fn get_config_from_args() -> Config {
    let mut args = env::args().skip(1);
    let mut config = Config::default();

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-v" => config.verbose = true,
            "-o" => config.output_file = Some(args.next().expect("no output file")),
            _ => config.input_files.push(arg),
        }
    }

    config
}

fn write_file(file: &str, unit: Unit) {
    match File::create(file.clone()) {
        Ok(mut file) => file.write_all(&unit.serialize().unwrap()).unwrap(),
        _ => panic!("could not open file `{}`", file),
    }
}

pub fn main() {
    let config = get_config_from_args();

    if config.input_files.is_empty() {
        panic!("no input files given");
    }

    let co = compiler::compile_files(&config.input_files).expect("could not compile code");

    write_file(&config.output_file(), co);
}
