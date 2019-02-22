use localc::vm::*;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn main()
{
    let args = env::args();
    let mut vm = Vm::with_stdlib();

    let stdin = io::stdin();

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
            panic!("could not open file `{}`", path);
        }
    }

    for line in stdin.lock().lines() {
        let script = line.unwrap();
        let program = vm.parser.parse(script.as_ref()).expect("parsing failed");
        println!("program: {:?}", program);
        let result = vm.run(&program);
        println!("result: {:?}", result);
    }
}
