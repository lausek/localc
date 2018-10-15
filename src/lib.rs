#![feature(box_patterns)]

pub mod parser;
pub mod program;

#[cfg(test)]
mod tests {
    use program::{*, node::Node};
    use parser::*;

    fn parse_str(script: &'static str)
        -> Result<Node, &'static str> 
    {
        parse(script.to_string())
    }

    fn exec_str(script: &'static str)
        -> Num
    {
        execute(&parse_str(script).unwrap()).unwrap()
    }

    #[test]
    fn addition() {
        assert_eq!(exec_str("18+18"), 36.0);
    }

    #[test]
    fn subtraction() {
        assert_eq!(exec_str("18-18"), 0.0);
    }

    #[test]
    fn multiplication() {
        assert_eq!(exec_str("18*18"), 324.0);
    }

    #[test]
    fn division() {
        assert_eq!(exec_str("18/18"), 1.0);
    }

    #[test]
    fn division_zero() {
        let program = parse_str("18/0").unwrap();
        assert!(execute(&program).is_err(), "division with zero is not possible");
    }

    #[test]
    fn parse_simple() {
        assert_eq!(exec_str("1+1"), 2.0);
    }

    #[test]
    fn parse_long() {
        assert_eq!(exec_str("1+1-1+1-1+1-1+1-1"), 1.0);
    }

    #[test]
    fn parse_simple_higher() {
        assert_eq!(exec_str("1*1"), 1.0);
    }

    #[test]
    fn parse_long_higher() {
        assert!(exec_str("2*5/2*5/2*5") == 62.5);
    }

    #[test]
    fn parse_complex() {
        assert!(exec_str("2+10/2-2*1+1") == 6.0);
    }

    #[test]
    fn parse_two_numbers() {
        assert!(parse_str("10 10").is_err(), "two numbers not allowed without operator");
    }

    #[test]
    fn parse_parens_simple() {
        assert_eq!(exec_str("10*(2+1)"), 30.0);
    }

    #[test]
    fn parse_parens_complex() {
        assert_eq!(exec_str("10*(2*(2+1)-1)-1"), 49.0);
    }
    
    #[test]
    fn parse_parens_incorrect() {
        assert!(parse_str("10*((2*(2+1)-1)-1").is_err(), "nesting is not valid");
    }

    #[test]
    fn parse_empty() {
        assert!(parse_str("(())").is_err(), "empty expression is an error");
    }

    #[test]
    fn parse_brackets_empty() {
        assert!(parse_str("[]").is_err(), "empty expression is an error");
    }

    #[test]
    fn parse_brackets_simple() {
        assert_eq!(exec_str("10*[2+1]"), 30.0);
    }

    #[test]
    fn parse_brackets_complex() {
        assert_eq!(exec_str("10*[2*(2+1)-1]-1"), 49.0);
    }
    
    #[test]
    fn parse_brackets_incorrect() {
        assert!(parse_str("10*[(2*(2+1)-1]]-1").is_err(), "nesting is not valid");
    }

    #[test]
    fn parse_power() {
        assert_eq!(exec_str("10^3"), 1000.0);
    }

    #[test]
    fn parse_pi() {
        // FIXME: should round a little
        assert_eq!(exec_str("pi"), 3.141592653589793);
    }

    #[test]
    fn parse_pi_multiply() {
        // FIXME: should round a little
        assert_eq!(exec_str("2*pi"), 6.283185307179586);
    }

    #[test]
    fn parse_assign() {
        assert_eq!(exec_str("x=10"), 10.0);
    }

    #[test]
    fn parse_assign_to_num() {
        let program = parse_str("2=10").unwrap();
        assert!(execute(&program).is_err(), "assignment to number is not allowed");
    }

    #[test]
    fn parse_assign_expression() {
        assert_eq!(exec_str("x=[(10*19)+10]*2"), 400.0);
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
