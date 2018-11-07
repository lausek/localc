#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(self_struct_ctor)]

extern crate regex;

pub mod parser;
pub mod program;

#[cfg(test)]
mod tests
{
    use parser::*;
    use program::{context::Context, node::Node, *};

    fn parse_str(script: &'static str) -> Result<Node, String>
    {
        parse(script)
    }

    fn exec_str_pre(script: &'static str) -> Result<f64, String>
    {
        // FIXME: execute optimized version of code here too
        //        and compare; panic if unequal
        execute(&parse_str(script).unwrap()).and_then(|n: Num| Ok(n.into()))
    }

    fn exec_str_pre_with_ctx(script: &'static str, ctx: &mut Context) -> Result<f64, String>
    {
        // FIXME: execute optimized version of code here too
        //        and compare; panic if unequal
        execute_with_ctx(&parse_str(script).unwrap(), ctx).and_then(|n: Num| Ok(n.into()))
    }

    fn exec_str(script: &'static str) -> f64
    {
        exec_str_pre(script).unwrap()
    }

    #[test]
    fn parse_simple()
    {
        // addition
        assert_eq!(exec_str("1+1"), 2.0);
        assert_eq!(exec_str("18+18"), 36.0);

        // subtraction
        assert_eq!(exec_str("18-18"), 0.0);

        // multiplication
        assert_eq!(exec_str("18*18"), 324.0);

        // division
        assert_eq!(exec_str("18/18"), 1.0);

        // power
        assert_eq!(exec_str("10^3"), 1000.0);

        // division with zero
        assert!(
            exec_str_pre("18/0").is_err(),
            "division with zero is not possible"
        );
    }

    #[test]
    fn parse_long()
    {
        // addition & subtraction
        assert_eq!(exec_str("1+1-1+1-1+1-1+1-1"), 1.0);

        // multiplication & division
        assert_eq!(exec_str("2*5/2*5/2*5"), 62.5);

        // mixed
        assert_eq!(exec_str("2+10/2-2*1+1"), 6.0);
        assert_eq!(exec_str("10*(2+1)"), 30.0);
        assert_eq!(exec_str("10*(2*(2+1)-1)-1"), 49.0);
        assert_eq!(exec_str("10*[2+1]"), 30.0);
        assert_eq!(exec_str("10*[2*(2+1)-1]-1"), 49.0);
    }

    #[test]
    fn parse_complex()
    {
        // FIXME: should round a little
        // constants
        assert_eq!(exec_str("pi"), 3.141592653589793);
        assert_eq!(exec_str("2*pi"), 6.283185307179586);

        // assignments
        assert_eq!(exec_str("x=10"), 10.0);
        assert_eq!(exec_str("x=[(10*19)+10]*2"), 400.0);
    }

    #[test]
    fn parse_errors()
    {
        // two numbers
        assert!(
            parse_str("10 10").is_err(),
            "two numbers not allowed without operator"
        );

        // two operators
        assert!(
            parse_str("* /").is_err(),
            "two operators not allowed without numbers"
        );

        // paren nesting incorrect
        assert!(
            parse_str("10*((2*(2+1)-1)-1").is_err(),
            "nesting is not valid"
        );
        assert!(
            parse_str("10*[(2*(2+1)-1]]-1").is_err(),
            "nesting is not valid"
        );

        // empty expression
        assert!(parse_str("[]").is_err(), "empty expression is an error");
        assert!(parse_str("(())").is_err(), "empty expression is an error");

        // assignments
        assert!(
            exec_str_pre("2=10").is_err(),
            "assignment to number is not allowed"
        );

        assert!(
            exec_str_pre("f(2)=1").is_err(),
            "only identifiers allowed in function assignment position"
        );

        assert!(
            exec_str_pre("f(y,2)=1").is_err(),
            "only identifiers allowed in function assignment position"
        );

        assert!(
            exec_str_pre("f(2)=x(1)").is_err(),
            "only identifiers allowed in function assignment position"
        );

        // function calls
        assert!(
            exec_str_pre("unknown()").is_err(),
            "unknown function called"
        );
        assert!(exec_str_pre("sqrt()").is_err(), "too few arguments");
        assert!(exec_str_pre("sqrt(16,16)").is_err(), "too many arguments");

        // valid identifiers
        assert!(parse_str("x").is_ok(), "invalid identifier");
        assert!(parse_str("test").is_ok(), "invalid identifier");
        assert!(parse_str("test1").is_ok(), "invalid identifier");
        assert!(parse_str("1test").is_err(), "invalid identifier");
        assert!(parse_str("f'").is_ok(), "invalid identifier");
        assert!(parse_str("10").is_ok(), "number is treated as identifier");
    }

    #[test]
    fn parse_default_functions()
    {
        // sqrt
        assert_eq!(exec_str("sqrt(16)"), 4.0);
        assert_eq!(exec_str("sqrt(64)"), 8.0);

        // sqrtn
        assert_eq!(f64::round(exec_str("sqrtn(3,64)")), 4.0);
        assert_eq!(f64::round(exec_str("sqrtn(5,3125)")), 5.0);

        // log
        assert_eq!(exec_str("log(2, 8)"), 3.0);
        assert_eq!(exec_str("log(10, 100)"), 2.0);

        // log2
        assert_eq!(exec_str("log2(8)"), 3.0);
        assert_eq!(exec_str("log2(16)"), 4.0);

        // ln
        assert_eq!(exec_str("ln(10)"), 2.302585092994046);
        assert_eq!(exec_str("ln(1)"), 0.0);
        assert_eq!(exec_str("ln(e)"), 1.0);
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_version1()
    {
        // reducing prefixes
        assert_eq!(exec_str("--1"), 1.0);

        // multiplication without parens
        assert_eq!(exec_str("3*-1"), -3.0);
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_dependencies()
    {
        assert!(
            exec_str_pre("x=1").is_ok(),
            "assignment from constant failed"
        );

        assert!(
            exec_str_pre("x=x").is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre("x=x+1").is_err(),
            "self assignment is an invalid operation"
        );

        let mut ctx = Context::default();
        exec_str_pre_with_ctx("x=y*3", &mut ctx);
        exec_str_pre_with_ctx("f(x)=x*3", &mut ctx);
        exec_str_pre_with_ctx("bar()=f(x)", &mut ctx);

        assert!(
            exec_str_pre("y=x-1").is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre("f(x)=f(x)").is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre("f(x)=1+f(x)").is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre("sqrt(x=x+1)").is_err(),
            "self assignment is an invalid operation"
        );
    }
}
