extern crate ansi_term;

use localc::*;

use ansi_term::Color::*;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn main() {
    env_logger::init();
    let args = env::args();
    let mut repl = repl::Repl::with_stdlib();

    for path in args.skip(1) {
        match File::open(path.clone()) {
            Ok(file) => {
                for line in BufReader::new(file).lines() {
                    let line = line.unwrap();
                    if line.is_empty() {
                        continue;
                    }
                    repl.run(&line).expect("error in runtime script");
                }
            }
            _ => println!("could not open file `{}`", Red.paint(path)),
        }
    }

    repl.repeat().unwrap();
}
