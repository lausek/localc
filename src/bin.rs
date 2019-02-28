extern crate ansi_term;

use ansi_term::Color::*;
use localc::{
    ast::{Expr, Value},
    vm::*,
};

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn pretty_print(expr: &Expr)
{
    if let Expr::Comp(op, lhs, rhs) = expr {
        print!("{}", Red.paint(format!("{}", op)));
        print!("(");
        pretty_print(lhs);
        print!(",");
        pretty_print(rhs);
        print!(")");
    } else {
        match expr {
            Expr::Value(val) => match val {
                Value::Numeric(n) => print!("{}", n),
                Value::Logical(l) => print!("{}", l),
                Value::Tuple(ls) | Value::Set(ls) => {
                    let mut it = ls.iter();
                    if let Some(first) = it.next() {
                        pretty_print(first);
                    }
                    for other in it.skip(1) {
                        print!(",");
                        pretty_print(other);
                    }
                }
                _ => print!("{:?}", val),
            },
            Expr::Ref(name) => print!("{}", Blue.paint(name)),
            Expr::Func(name, args) => {
                print!("{}", Green.paint(name));
                print!("(");
                if let Some(args) = args {
                    for arg in args {
                        pretty_print(arg);
                    }
                }
                print!(")");
            }
            _ => unreachable!(),
        }
    }
}

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
    vm: Vm,
    pub print_parse: bool,
}

impl Repl
{
    pub fn new(vm: Vm) -> Self
    {
        Self {
            vm,
            print_parse: true,
        }
    }

    pub fn repeat(&mut self) -> Result<(), String>
    {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let script = line.unwrap();
            let result = self.vm.parser.parse(script.as_ref());
            if self.print_parse {
                println!("parsed: {:?}", result);
            }
            if let Ok(mut program) = result {
                if self.vm.config().is_optimizing() {
                    self.vm.optimize(&mut program)?;
                }
                print!(
                    "program{}: ",
                    Yellow.paint(if self.vm.config().is_optimizing() {
                        " [optimized]"
                    } else {
                        ""
                    }),
                );
                pretty_print(&program);
                println!();
                present!(self.vm.run(&program));
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

    Repl::new(vm).repeat().unwrap();
}
