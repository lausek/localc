extern crate localc;

use std::fs::File;

fn exec(script: &str)
{
    /*
    match localc::parser::parse(script.to_string()) {
        Ok(program) => match localc::program::execute(&program) {
            Ok(result) => println!("{}", result),
            _ => {}
        },
        Err(msg) => println!("{:?}", msg),
    }
    */
}

pub fn main()
{
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    let parser = localc::query::ExprParser::new();

    for line in stdin.lock().lines() {
        let script = line.unwrap();
        println!("{:?}", parser.parse(script.as_ref()));
    }

    /*
    use localc::program::context::Context;
    use std::env;
    use std::io::{self, BufRead};
    
    let mut arg_iter = env::args();
    let mut executed = 0;
    let mut vcompile = false;
    let mut vparse = false;
    let mut pidents = false;
    let mut nodefault = false;
    
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
                println!("{:?}", localc::program::execute_script(file.unwrap()));
                executed += 1;
            }
            "-no-default" => {
                nodefault = true;
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
    
    let mut ctx = if nodefault {
        Context::new()
    } else {
        Context::default()
    };
    
    if executed == 0 {
        let stdin = io::stdin();
    
        for line in stdin.lock().lines() {
            if let Ok(script) = line {
                if vparse {
                    println!("{:?}", localc::parser::lexer::tokenize(script));
                } else {
                    match localc::parser::parse(script.to_string()) {
                        Ok(program) => {
                            if vcompile {
                                println!("{:?}", program);
                            } else {
                                println!(
                                    "{:?}",
                                    localc::program::execute_with_ctx(&program, &mut ctx)
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
    //}
    */
}
