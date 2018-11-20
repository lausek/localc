extern crate treecalc;

use std::fs::File;

fn exec(script: &str)
{
    match treecalc::parser::parse(script.to_string()) {
        Ok(program) => {
            println!("{:?}", treecalc::program::execute(&program));
        }
        Err(msg) => println!("{:?}", msg),
    }
}

pub fn main()
{
    use std::env;
    use std::io::{self, BufRead};
    use treecalc::program::context::Context;
    
    let mut ctx = Context::default();

    let mut arg_iter = env::args();
    let mut executed = 0;
    let mut vcompile = false;
    let mut vparse = false;
    let mut pidents = false;

    let next_arg = |it: &mut std::env::Args| {
        let expression = it.next();
        if expression.is_none() {
            panic!("parameter expected, got nothing");
        }
        expression.unwrap()
    };

    while let Some(arg) = arg_iter.next() {
        match arg.as_str() {
            "-e" => {
                exec(&next_arg(&mut arg_iter));
                executed += 1;
            }
            "-s" => {
                let path = next_arg(&mut arg_iter);
                let file = File::open(path.clone());
                if let Err(msg) = file {
                    panic!(format!("{}: {}", msg, path));
                }
                println!("{:?}", treecalc::program::execute_script(file.unwrap()));
                executed += 1;
            }
            // `vad` does this compile to?
            "-vc" => {
                vcompile = true;
            }
            // `vad` does this parse as?
            "-vp" => {
                vparse = true;
            }
            // print identifiers
            "-pi" => {
                pidents = true;
            }
            _ => {}
        }
    }

    if executed == 0 {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {
            if let Ok(script) = line {
                if vparse {
                    println!("{:?}", treecalc::parser::lexer::tokenize(script));
                } else {
                    match treecalc::parser::parse(script.to_string()) {
                        Ok(program) => {
                            if vcompile {
                                println!("{:?}", program);
                            } else {
                                println!(
                                    "{:?}",
                                    treecalc::program::execute_with_ctx(&program, &mut ctx)
                                );
                                println!("\nContext:\n{}", ctx);
                            }
                            if pidents {
                                println!("{:?}", program.idents());
                            }
                        }
                        Err(msg) => println!("{:?}", msg),
                    }
                }
            }
        }
    }
}
