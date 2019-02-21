#![feature(box_patterns)]
#![feature(box_syntax)]
#![allow(illegal_floating_point_literal_pattern)]

extern crate lazy_static;
extern crate rand;
extern crate regex;

#[macro_use]
extern crate lalrpop_util;

pub mod ast;
pub mod vm;

lalrpop_mod!(pub query);

/*
#[cfg(test)]
mod tests
{
    #[test]
    fn parsing()
    {
        use crate::ast::{Expr::*, Value::*};

        let parser = super::query::ExprParser::new();

        match parser.parse("22") {
            Ok(Value(Numeric(22.0))) => {}
            _ => assert!(false),
        }

        match parser.parse("-22") {
            Ok(Value(Numeric(-22.0))) => {}
            _ => assert!(false),
        }

        // TODO: write more test
    }
}
*/

#[cfg(test)]
mod tests
{
    use crate::{
        ast::{Value::*, *},
        query::*,
        vm::*,
    };

    macro_rules! matches {
        ($script:expr, $m:pat) => {
            matches!(Vm::new(), $script, $m);
        };
        ($vm:expr, $script:expr, $m:pat) => {
            match $vm.run(&ExprParser::new().parse($script).unwrap()) {
                $m => assert!(true),
                _ => assert!(false),
            }
        };
    }

    fn parse_str(script: &'static str) -> Result<Expr, String>
    {
        match ExprParser::new().parse(script) {
            Ok(expr) => Ok(expr),
            _ => Err("an error occurred".to_string()),
        }
    }

    #[test]
    fn parse_simple()
    {
        // addition
        matches!("1 + 1", Ok(Numeric(2.0)));
        matches!("18 + 18", Ok(Numeric(36.0)));

        // subtraction
        matches!("18 - 18", Ok(Numeric(0.0)));

        // multiplication
        matches!("18 * 18", Ok(Numeric(324.0)));

        // division
        matches!("18 / 18", Ok(Numeric(1.0)));

        // power
        matches!("10 ^ 3", Ok(Numeric(1000.0)));

        // modulo
        matches!("8 % 2", Ok(Numeric(0.0)));
        matches!("9 % 2", Ok(Numeric(1.0)));

        // division with zero
        matches!("18 / 0", Err(_));
    }

    #[test]
    fn parse_long()
    {
        // addition & subtraction
        matches!("1+1-1+1-1+1-1+1-1", Ok(Numeric(1.0)));

        // multiplication & division
        matches!("2*5/2*5/2*5", Ok(Numeric(62.5)));

        // mixed
        matches!("2+10/2-2*1+1", Ok(Numeric(6.0)));
        matches!("10*(2+1)", Ok(Numeric(30.0)));
        matches!("10*(2*(2+1)-1)-1", Ok(Numeric(49.0)));
        matches!("10*[2+1]", Ok(Numeric(30.0)));
        matches!("10*[2*(2+1)-1]-1", Ok(Numeric(49.0)));
    }

    #[test]
    fn parse_complex()
    {
        // FIXME: should round a little
        // constants
        matches!("pi", Ok(Numeric(3.141592653589793)));
        matches!("pi * 2", Ok(Numeric(6.283185307179586)));

        // assignments
        matches!("x = 10", Ok(Empty));
        //matches!("x = [(10 * 19) + 10] * 2", Ok(Empty));
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
        assert!(
            parse_str("10*{(2*(2+1)-1)-1").is_err(),
            "nesting is not valid"
        );

        // empty expression
        assert!(parse_str("[]").is_err(), "empty expression is an error");
        assert!(parse_str("(())").is_err(), "empty expression is an error");

        // assignments
        matches!("2 = 10", Err(_));

        /*
         * TODO: these won't be errors in near future
        assert!(
            exec_str_pre_num("f(2)=1").is_err(),
            "only identifiers allowed in function assignment position"
        );
        
        assert!(
            exec_str_pre_num("f(y,2)=1").is_err(),
            "only identifiers allowed in function assignment position"
        );
        
        assert!(
            exec_str_pre_num("f(2)=x(1)").is_err(),
            "only identifiers allowed in function assignment position"
        );
        */

        // function calls
        matches!("unknown()", Err(_));
        matches!("sqrt()", Err(_));
        matches!("sqrt(16, 16)", Err(_));

        // valid identifiers
        assert!(parse_str("x").is_ok(), "invalid identifier");
        assert!(parse_str("test").is_ok(), "invalid identifier");
        assert!(parse_str("test1").is_ok(), "invalid identifier");
        assert!(parse_str("1test").is_err(), "invalid identifier");
        assert!(parse_str("f'").is_ok(), "invalid identifier");
        assert!(parse_str("10").is_ok(), "number is treated as identifier");
        assert!(parse_str("empty?").is_ok(), "invalid identifier");
        assert!(parse_str("empt?y").is_err(), "invalid identifier");
    }

    #[test]
    fn parse_default_functions()
    {
        // sqrt
        matches!("sqrt(16)", Ok(Numeric(4.0)));
        matches!("sqrt(64)", Ok(Numeric(8.0)));

        // sqrtn
        // matches!(f64::round(exec_str("sqrtn(3,64)")), Ok(Numeric(4.0)));
        // matches!(f64::round(exec_str("sqrtn(5,3125)")), Ok(Numeric(5.0)));

        // log
        matches!("log(2, 8)", Ok(Numeric(3.0)));
        matches!("log(10, 100)", Ok(Numeric(2.0)));

        // log2
        matches!("log2(8)", Ok(Numeric(3.0)));
        matches!("log2(16)", Ok(Numeric(4.0)));

        // ln
        matches!("ln(10)", Ok(Numeric(2.302585092994046)));
        matches!("ln(1)", Ok(Numeric(0.0)));
        matches!("ln(e)", Ok(Numeric(1.0)));

        // if
        matches!("if(1==1,1,2)", Ok(Numeric(1.0)));
        matches!("if(1!=1,1,2)", Ok(Numeric(2.0)));
        /*
        assert!(
            exec_str_pre_num("if(1,1,2)").is_err(),
            "only logical values allowed in `if` condition"
        );
        */

        // empty?
        matches!("empty?({})", Ok(Logical(true)));
        matches!("empty?({1})", Ok(Logical(false)));
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_version1()
    {
        // reducing prefixes
        matches!("--1", Ok(Numeric(1.0)));

        // multiplication without parens
        matches!("3*-1", Ok(Numeric(-3.0)));

        matches!("-(1+2)", Ok(Numeric(-3.0)));
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_truth()
    {
        // equal, not equal
        matches!("1==1", Ok(Logical(true)));
        matches!("1!=1", Ok(Logical(false)));

        // ordering
        matches!("20>10", Ok(Logical(true)));
        matches!("10<10", Ok(Logical(false)));
        matches!("10<1020", Ok(Logical(true)));
        matches!("10>1020", Ok(Logical(false)));

        matches!("10<=10", Ok(Logical(true)));
        matches!("10<=11", Ok(Logical(true)));
        matches!("10>=5", Ok(Logical(true)));
        matches!("10>=10", Ok(Logical(true)));

        // or
        matches!("1==1 || 2==2", Ok(Logical(true)));
        matches!("1==1 || 2!=2", Ok(Logical(true)));
        matches!("1!=1 || 2==2", Ok(Logical(true)));
        matches!("1!=1 || 2!=2", Ok(Logical(false)));

        // and
        matches!("1==1 && 2==2", Ok(Logical(true)));
        matches!("1==1 && 2!=2", Ok(Logical(false)));
        matches!("1!=1 && 2==2", Ok(Logical(false)));
        matches!("1!=1 && 2!=2", Ok(Logical(false)));
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_tuple()
    {
        // parsing
        matches!(exec_str_set("{1,2,3}"), vec!["1", "2", "3"]);
        matches!(exec_str_set("{log(2, 4), 2}"), vec!["2", "2"]);
        matches!(exec_str_set("{}"), Vec::<String>::new());

        // indexing
        matches!(exec_str("{1,2,3}_2"), 3.0);
        matches!(exec_str("{1,2,3}_2^2"), 9.0);
        assert!(
            exec_str_pre_set("{1,2,3}_(1==2)^2").is_err(),
            "bool is not a valid index"
        );

        // generator
        matches!(exec_str_set("{x | 0 < x, x < 5}"), vec!["1", "2", "3", "4"]);
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_dependencies()
    {
        assert!(
            exec_str_pre_truth("x=1").is_ok(),
            "assignment from constant failed"
        );

        assert!(
            exec_str_pre_truth("1=2").is_err(),
            "assignment to constant is an invalid operation"
        );

        assert!(
            exec_str_pre_truth("x=x").is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre_truth("x=x+1").is_err(),
            "self assignment is an invalid operation"
        );

        let mut vm = Vm::new();
        matches!(vm, "x = y * 3", Ok(Empty));
        matches!(vm, "f(x) = x * 3", Ok(Empty));
        matches!(vm, "bar() = f(x)", Ok(Empty));

        assert!(
            exec_str_pre_with_vm("y=x-1", &mut vm).is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre_with_vm("f(x)=f(x)", &mut vm).is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre_with_vm("f(x)=1+f(x)", &mut vm).is_err(),
            "self assignment is an invalid operation"
        );

        assert!(
            exec_str_pre_with_vm("sqrt(x=x+1)", &mut vm).is_err(),
            "self assignment is an invalid operation"
        );
    }
}
