mod program;
mod parser;

fn exec(script: String)
{
    match parser::parse(script) {
        Ok(program) => {
            println!("{:?}", program::execute(&program));
        },
        Err(msg) => println!("{:?}", msg),
    }
}

pub fn main()
{
    use std::env;
    use std::io::{self, BufRead};
  
    let mut arg_iter = env::args();
    let mut executed = 0;

    while let Some(arg) = arg_iter.next() {
        match arg.as_str() {
            "-e" => {
                let expression = arg_iter.next();
                if expression.is_none() {
                    break;
                }
                exec(expression.unwrap());
                executed += 1;
            },
            _ => {},
        }
    }

    if executed == 0 {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {
            if let Ok(script) = line {
                exec(script);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use program::{*, Node::*};
    use parser::*;

    #[test]
    fn addition() {
        let program = Add(
            Box::new(Value(18.0)),
            Box::new(Value(18.0))
            );
        assert_eq!(execute(&program).unwrap(), 36.0);
    }

    #[test]
    fn subtraction() {
        let program = Sub(
            Box::new(Value(18.0)),
            Box::new(Value(18.0))
            );
        assert_eq!(execute(&program).unwrap(), 0.0);
    }

    #[test]
    fn multiplication() {
        let program = Mul(
            Box::new(Value(18.0)),
            Box::new(Value(18.0))
            );
        assert_eq!(execute(&program).unwrap(), 324.0);
    }

    #[test]
    fn division() {
        let program = Div(
            Box::new(Value(18.0)),
            Box::new(Value(18.0))
            );
        assert_eq!(execute(&program).unwrap(), 1.0);
    }

    #[test]
    fn division_zero() {
        let program = Div(
            Box::new(Value(18.0)),
            Box::new(Value(0.0))
            );
        assert!(execute(&program).is_err(), "division with zero should not be possible");
    }

    #[test]
    fn parse_simple() {
        let script = String::from("1+1");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 2.0);
    }

    #[test]
    fn parse_long() {
        let script = String::from("1+1-1+1-1+1-1+1-1");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 1.0);
    }

    #[test]
    fn parse_simple_higher() {
        let script = String::from("1*1");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 1.0);
    }

    #[test]
    fn parse_long_higher() {
        let script = String::from("2*5/2*5/2*5");
        let program = parse(script).unwrap();
        assert!(execute(&program).unwrap() == 62.5, format!("{:?}", program));
    }

    #[test]
    fn parse_complex() {
        let script = String::from("2+10/2-2*1+1");
        let program = parse(script).unwrap();
        assert!(execute(&program).unwrap() == 6.0, format!("{:?}", program));
    }

    #[test]
    fn parse_two_numbers() {
        let script = String::from("10 10");
        let program = parse(script);
        assert!(program.is_err(), "two numbers not allowed without operator");
    }

    #[test]
    fn parse_parens_simple() {
        let script = String::from("10*(2+1)");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 30.0);
    }

    #[test]
    fn parse_parens_complex() {
        let script = String::from("10*(2*(2+1)-1)-1");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 49.0);
    }
    
    #[test]
    fn parse_parens_incorrect() {
        let script = String::from("10*((2*(2+1)-1)-1");
        let program = parse(script);
        println!("{:?}", program);
        assert!(program.is_err(), "nesting is not valid");
    }

    #[test]
    fn parse_empty() {
        let script = String::from("(())");
        assert!(parse(script).is_err(), "empty expression is an error");
    }

    #[test]
    fn parse_brackets_empty() {
        let script = String::from("[]");
        assert!(parse(script).is_err(), "empty expression is an error");
    }

    #[test]
    fn parse_brackets_simple() {
        let script = String::from("10*[2+1]");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 30.0);
    }

    #[test]
    fn parse_brackets_complex() {
        let script = String::from("10*[2*(2+1)-1]-1");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 49.0);
    }
    
    #[test]
    fn parse_brackets_incorrect() {
        let script = String::from("10*[(2*(2+1)-1]]-1");
        let program = parse(script);
        println!("{:?}", program);
        assert!(program.is_err(), "nesting is not valid");
    }

    #[test]
    fn parse_power() {
        let script = String::from("10^3");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 1000.0);
    }

    /*
    #[test]
    fn parse_function() {
        let script = String::from("sqrt(16)");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 4.0);
    }
    */
}
