mod program;
mod parser;

pub fn main()
{
    use std::io::{self, BufRead};
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if let Ok(script) = line {
            match parser::parse(script) {
                Ok(program) => {
                    println!("{:?}", program::execute(&program));
                },
                Err(msg) => println!("{:?}", msg),
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
}
