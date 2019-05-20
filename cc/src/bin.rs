use localc::*;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn get_program_args() -> (Vec<String>, Option<String>) {
    let mut args = env::args().skip(1);
    let mut files = vec![];
    let mut output = None;

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-o" => {
                output = Some(args.next().expect("no output file"));
            }
            _ => files.push(arg),
        }
    }

    (files, output)
}

fn write_file(file: String, module: Module) {
    match File::create(file.clone()) {
        Ok(mut file) => file.write_all(&module.serialize().unwrap()).unwrap(),
        _ => panic!("could not open file `{}`", file),
    }
}

pub fn main() {
    let (files, output) = get_program_args();
    let co = compiler::compile_files(&files).expect("could not compile code");

    let output = output.unwrap_or("out.lcc".to_string());

    write_file(output, co);
}
