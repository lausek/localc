#![feature(box_patterns)]
#![feature(box_syntax)]
#![allow(clippy::all)]

extern crate env_logger;
#[macro_use]
extern crate lalrpop_util;
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rand;
extern crate regex;

pub mod ast;
pub mod compiler;
pub mod vm;

lalrpop_mod!(pub expr);

#[cfg(test)]
mod tests
{
    use crate::{
        ast::{Value::*, *},
        expr::*,
        vm::*,
    };

    macro_rules! eq {
        ($script:expr, $ex:expr) => {
            eq!(Vm::new(), $script, $ex);
        };
        ($vm:expr, $script:expr, $ex:expr) => {
            let result = $vm.run_expr(&$vm.parser.parse($script).unwrap());
            assert_eq!(result, $ex);
        };
    }

    macro_rules! err {
        ($script:expr, $msg:expr) => {
            err!(Vm::new(), $script, $msg);
        };
        ($vm:expr, $script:expr, $msg:expr) => {
            assert!(
                $vm.run_expr(&$vm.parser.parse($script).unwrap()).is_err(),
                $msg
            );
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
    fn test_numeric()
    {
        // addition
        eq!("1 + 1", Ok(Numeric(2.)));
        eq!("18 + 18", Ok(Numeric(36.)));

        // subtraction
        eq!("18 - 18", Ok(Numeric(0.)));

        // multiplication
        eq!("18 * 18", Ok(Numeric(324.)));

        // division
        eq!("18 / 18", Ok(Numeric(1.)));

        // power
        eq!("10 ^ 3", Ok(Numeric(1000.)));

        // modulo
        eq!("8 % 2", Ok(Numeric(0.)));
        eq!("9 % 2", Ok(Numeric(1.)));

        // division with zero
        err!("18 / 0", "division by zero not allowed");

        // different priority
        eq!("1 + 4 * 5", Ok(Numeric(21.)));
        eq!("(1 + 4) * 5", Ok(Numeric(25.)));
        eq!("(1 + 4) * 5 / 2", Ok(Numeric(12.5)));

        // addition & subtraction
        eq!("1 + 1 - 1 + 1 - 1 + 1 - 1 + 1 - 1", Ok(Numeric(1.)));

        // multiplication & division
        eq!("2 * 5 / 2 * 5 / 2 * 5", Ok(Numeric(62.5)));

        // mixed
        eq!("2 + 10 / 2 - 2 * 1 + 1", Ok(Numeric(6.)));
        eq!("10 * (2 + 1)", Ok(Numeric(30.)));
        eq!("10 * (2 * (2 + 1) - 1) - 1", Ok(Numeric(49.)));

        // TODO:
        //eq!("10 * [2 + 1]", Ok(Numeric(30.0)));
        //eq!("10 * [2*(2 + 1) - 1] - 1", Ok(Numeric(49.0)));
        // reducing prefixes
        //eq!("--1", Ok(Numeric(1.0)));
        // multiplication without parens
        //eq!("3*-1", Ok(Numeric(-3.0)));
        //eq!("-(1+2)", Ok(Numeric(-3.0)));
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
        err!("2 = 10", "assignment to constant not allowed");

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
        err!("unknown()", "invalid call");
        err!("sqrt()", "invalid call");
        err!("sqrt(16, 16)", "invalid call");

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
    fn test_stdlib()
    {
        let mut vm = Vm::with_stdlib();

        // constants
        eq!(vm, "pi", Ok(Numeric(std::f64::consts::PI)));
        eq!(vm, "e", Ok(Numeric(std::f64::consts::E)));

        // sqrt
        eq!(vm, "sqrt(16)", Ok(Numeric(4.)));
        eq!(vm, "sqrt(64)", Ok(Numeric(8.)));

        // sqrtn
        //eq!(vm, "sqrt(64, 3)", Ok(Numeric(4.)));
        //eq!(vm, "sqrt(3125, 5)", Ok(Numeric(5.)));

        // log
        eq!(vm, "log(8, 2)", Ok(Numeric(3.)));
        eq!(vm, "log(100, 10)", Ok(Numeric(2.)));
        eq!(vm, "log(100)", Ok(Numeric(2.)));

        // ln
        //eq!(vm, "ln(10)", Ok(Numeric(2.302585092994046)));
        //eq!(vm, "ln(1)", Ok(Numeric(0.0)));
        //eq!(vm, "ln(e)", Ok(Numeric(1.0)));

        // if
        eq!(vm, "if(1==1,1,2)", Ok(Numeric(1.)));
        eq!(vm, "if(1!=1,1,2)", Ok(Numeric(2.)));

        // assert
        eq!(vm, "assert(1==1)", Ok(Nil));
        eq!(vm, "assert(true)", Ok(Nil));

        // empty?
        //eq!(vm, "empty?({})", Ok(Logical(true)));
        //eq!(vm, "empty?({1})", Ok(Logical(false)));
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_version1() {}

    #[test]
    fn test_logical()
    {
        // constants
        eq!("true", Ok(Logical(true)));
        eq!("false", Ok(Logical(false)));

        // equal, not equal
        eq!("1==1", Ok(Logical(true)));
        eq!("1!=1", Ok(Logical(false)));

        // ordering
        eq!("20>10", Ok(Logical(true)));
        eq!("10<10", Ok(Logical(false)));
        eq!("10<1020", Ok(Logical(true)));
        eq!("10>1020", Ok(Logical(false)));

        eq!("10<=10", Ok(Logical(true)));
        eq!("10<=11", Ok(Logical(true)));
        eq!("10>=5", Ok(Logical(true)));
        eq!("10>=10", Ok(Logical(true)));

        // or
        eq!("1==1 || 2==2", Ok(Logical(true)));
        eq!("1==1 || 2!=2", Ok(Logical(true)));
        eq!("1!=1 || 2==2", Ok(Logical(true)));
        eq!("1!=1 || 2!=2", Ok(Logical(false)));

        // and
        eq!("1==1 && 2==2", Ok(Logical(true)));
        eq!("1==1 && 2!=2", Ok(Logical(false)));
        eq!("1!=1 && 2==2", Ok(Logical(false)));
        eq!("1!=1 && 2!=2", Ok(Logical(false)));
    }

    #[test]
    fn context_example()
    {
        let mut vm = Vm::with_stdlib();
        eq!(vm, "even(n) = n % 2 == 0", Ok(Nil));
        eq!(vm, "even(1)", Ok(Logical(false)));
        eq!(vm, "even(2)", Ok(Logical(true)));
        eq!(vm, "odd(n) = even(n) == (1 != 1)", Ok(Nil));
        eq!(vm, "odd(1)", Ok(Logical(true)));
        eq!(vm, "odd(2)", Ok(Logical(false)));
    }

    #[test]
    fn test_optimize()
    {
        use crate::ast::Expr::*;
        let vm = Vm::with_stdlib();
        let parse_optimized = |vm: &Vm, s: &str| -> Option<Expr> {
            match vm.parser.parse(s) {
                Ok(mut program) => {
                    vm.optimize(&mut program).unwrap();
                    Some(program)
                }
                _ => unimplemented!(),
            }
        };

        match parse_optimized(&vm, "1 + 2 + 3") {
            Some(Value(Numeric(n))) if n == 6. => {}
            _ => panic!("optimization wrong"),
        }

        match parse_optimized(&vm, "1 != 1 && 1 == 1") {
            Some(Value(Logical(false))) => {}
            _ => panic!("optimization wrong"),
        }

        match parse_optimized(&vm, "1 == 1 || 1 != 1") {
            Some(Value(Logical(true))) => {}
            _ => panic!("optimization wrong"),
        }

        match parse_optimized(&vm, "x * 0") {
            Some(Value(Numeric(n))) if n == 0. => {}
            _ => panic!("optimization wrong"),
        }
    }

    #[cfg(feature = "v1-0")]
    #[test]
    fn test_tuple()
    {
        // parsing
        eq!(
            "{1, 2, 3}",
            Ok(Value::Set(vec![
                Expr::Value(Value::Numeric(1.)),
                Expr::Value(Value::Numeric(2.)),
                Expr::Value(Value::Numeric(3.)),
            ]))
        );
        eq!(
            "{log(2, 4), 2}",
            Ok(Value::Set(vec![
                Expr::Value(Value::Numeric(2.)),
                Expr::Value(Value::Numeric(2.)),
            ]))
        );
        eq!("{}", Ok(Value::Set(vec![])));

        /*
                    // indexing
                    eq!(exec_str("{1,2,3}_2"), 3.0);
                    eq!(exec_str("{1,2,3}_2^2"), 9.0);
                    assert!(
                        exec_str_pre_set("{1,2,3}_(1==2)^2").is_err(),
                        "bool is not a valid index"
                    );

                    // generator
                    eq!(exec_str_set("{x | 0 < x, x < 5}"), vec!["1", "2", "3", "4"]);
        */
    }

    /*
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
            eq!(vm, "x = y * 3", Ok(Nil));
            eq!(vm, "f(x) = x * 3", Ok(Nil));
            eq!(vm, "bar() = f(x)", Ok(Nil));

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
    */
}
