extern crate ansi_term;

use ansi_term::Color::*;
use localc::vm::*;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

macro_rules! present {
    ($result:expr) => {
        let s = format!("{:?}", $result);
        if $result.is_ok() {
            println!("{}", Green.paint(s));
        } else {
            println!("{}", Red.paint(s));
        }
    };
}

struct Repl
{
    pub optimize: bool,
}

impl Repl
{
    pub fn new() -> Self
    {
        Self { optimize: true }
    }

    pub fn repeat(&self, vm: &mut Vm) -> Result<(), String>
    {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let script = line.unwrap();
            let result = vm.parser.parse(script.as_ref());
            if let Ok(mut program) = result {
                if self.optimize {
                    vm.optimize(&mut program)?;
                }
                println!(
                    "program{}: {:?}",
                    if self.optimize { " [optimized]" } else { "" },
                    program
                );
                present!(vm.run(&program));
            } else {
                present!(result);
            }
        }
        Ok(())
    }
}

pub fn main()
{
    env_logger::init();
    let args = env::args();
    let mut vm = Vm::with_stdlib();

    for path in args.skip(1) {
        if let Ok(file) = File::open(path.clone()) {
            for line in BufReader::new(file).lines() {
                let line = line.unwrap();
                if line.is_empty() {
                    continue;
                }
                vm.run_raw(line.as_ref()).expect("error in runtime script");
            }
        } else {
            println!("could not open file `{}`", Red.paint(path));
        }
    }

    Repl::new().repeat(&mut vm).unwrap();
}
