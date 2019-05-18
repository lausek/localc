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
    let mut repl = repl::Repl::with_stdlib();

    let (files, output) = get_program_args();

    for file in files.iter() {
        match File::open(file.clone()) {
            Ok(file) => {
                for line in BufReader::new(file).lines() {
                    let line = line.unwrap();
                    if line.is_empty() {
                        continue;
                    }
                    repl.run(&line).expect("error in runtime script");
                }
            }
            _ => panic!("could not open file `{}`", file),
        }
    }

    let output = output.unwrap_or("out.llc".to_string());
    let co = repl.runtime.module.build().unwrap();

    write_file(output, co);
}
