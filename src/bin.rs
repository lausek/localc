extern crate treecalc;

fn exec(script: &str)
{
    match treecalc::parser::parse(script) {
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
    use treecalc::program::context::GenericContext;

    let mut arg_iter = env::args();
    let mut executed = 0;
    let mut vcompile = false;
    let mut vparse = false;

    while let Some(arg) = arg_iter.next() {
        match arg.as_str() {
            "-e" => {
                let expression = arg_iter.next();
                if expression.is_none() {
                    break;
                }
                exec(&expression.unwrap());
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
            _ => {}
        }
    }

    if executed == 0 {
        let stdin = io::stdin();
        let mut ctx: GenericContext = Default::default();

        for line in stdin.lock().lines() {
            if let Ok(script) = line {
                if vparse {
                    println!("{:?}", treecalc::parser::lexer::tokenize(script.as_str()));
                } else {
                    match treecalc::parser::parse(script.as_str()) {
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
                        }
                        Err(msg) => println!("{:?}", msg),
                    }
                }
            }
        }
    }
}
