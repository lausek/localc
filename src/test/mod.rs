#![cfg(test)]

use super::*;

use lovm::gen;
use lovm::vm;

#[test]
fn building_fib() {
    // let's build the fibonacci function
    let mut repl = Repl::new();

    repl.run("f(0) = 0").unwrap();
    repl.run("f(1) = 1").unwrap();
    repl.run("f(x) = f(x - 1) + f(x - 2)").unwrap();

    fn debug(data: &mut vm::VmData) -> vm::VmResult {
        let frame = data.stack.last_mut().unwrap();
        println!("{:?}", frame);
        let result = data.vstack.pop().expect("no value");
        assert_eq!(result, lovm::Value::I64(0));
        Ok(())
    }

    repl.runtime
        .vm
        .interrupts_mut()
        .set(vm::Interrupt::Debug as usize, &debug);

    repl.run("f(8)").unwrap();

    /*
    let mut func = crate::runtime::func::Function::new();

    let mut v0 = gen::FunctionBuilder::new();
    v0.step(gen::Operation::ret().op(0).end());

    let mut v1 = gen::FunctionBuilder::new();
    v1.step(gen::Operation::ret().op(1).end());

    func.overload(vec![0f64], v0.clone());
    func.overload(vec![1f64], v1.clone());
    //func.overload(vec!["x"], v1);

    println!("{}", func);
    let result = func.build().unwrap();
    println!("{}", result);
    */
}

#[test]
fn building_sqrt() {
    let mut func = crate::runtime::func::Function::new();

    let mut a1 = gen::FunctionBuilder::new();
    a1.step(
        gen::Operation::ret()
            .op(gen::Operation::pow().op(0.5).end())
            .end(),
    );

    let mut a2 = gen::FunctionBuilder::new();
    a2.step(gen::Operation::pop().var("n").end());
    let exp = gen::Operation::div().op(1).var("n").end();
    a2.step(
        gen::Operation::ret()
            .op(gen::Operation::pow().op(exp).end())
            .end(),
    );

    func.overload(vec!["x"], a1);
    func.overload(vec!["x", "n"], a2);

    println!("{}", func);
    let result = func.build();

    assert!(false);
}

#[test]
fn building_multiargs() {
    // implements crazy logic
    // return false if (0, 1)
    // return true if (1, 0)
    // return `eq` if (x, y)
    let mut func = crate::runtime::func::Function::new();

    let mut v0 = gen::FunctionBuilder::new();
    v0.step(gen::Operation::ret().op(false).end());

    let mut v1 = gen::FunctionBuilder::new();
    v1.step(gen::Operation::ret().op(true).end());

    let mut v2 = gen::FunctionBuilder::new();
    v2.step(
        gen::Operation::ret()
            .op(gen::Operation::cmp_eq().var("x").var("y").end())
            .end(),
    );

    func.overload(vec![0f64, 1f64], v0.clone());
    func.overload(vec![1f64, 0f64], v1.clone());
    func.overload(vec!["x", "y"], v2.clone());

    println!("{}", func);
    let result = func.build();
    println!("{}", result.unwrap());

    assert!(false);
}

//use crate::{expr::*, repl::*};
//
//macro_rules! eq {
//    ($script:expr, $ex:expr) => {
//        eq!(Repl::new(), $script, $ex);
//    };
//    ($repl:expr, $script:expr, $ex:expr) => {
//        let result = $repl.run_expr(&$repl.parser.parse($script).unwrap());
//        assert_eq!(result, $ex);
//    };
//}
//
//macro_rules! err {
//    ($script:expr, $msg:expr) => {
//        err!(Repl::new(), $script, $msg);
//    };
//    ($repl:expr, $script:expr, $msg:expr) => {
//        assert!(
//            $repl
//                .run_expr(&$repl.parser.parse($script).unwrap())
//                .is_err(),
//            $msg
//        );
//    };
//}
//
//fn parse_str(script: &'static str) -> Result<Expr, String> {
//    match ExprParser::new().parse(script) {
//        Ok(expr) => Ok(expr),
//        _ => Err("an error occurred".to_string()),
//    }
//}
//
//#[test]
//fn test_numeric() {
//    // addition
//    eq!("1 + 1", Ok(Numeric(2.)));
//    eq!("18 + 18", Ok(Numeric(36.)));
//
//    // subtraction
//    eq!("18 - 18", Ok(Numeric(0.)));
//
//    // multiplication
//    eq!("18 * 18", Ok(Numeric(324.)));
//
//    // division
//    eq!("18 / 18", Ok(Numeric(1.)));
//
//    // power
//    eq!("10 ^ 3", Ok(Numeric(1000.)));
//
//    // modulo
//    eq!("8 % 2", Ok(Numeric(0.)));
//    eq!("9 % 2", Ok(Numeric(1.)));
//
//    // division with zero
//    err!("18 / 0", "division by zero not allowed");
//
//    // different priority
//    eq!("1 + 4 * 5", Ok(Numeric(21.)));
//    eq!("(1 + 4) * 5", Ok(Numeric(25.)));
//    eq!("(1 + 4) * 5 / 2", Ok(Numeric(12.5)));
//
//    // addition & subtraction
//    eq!("1 + 1 - 1 + 1 - 1 + 1 - 1 + 1 - 1", Ok(Numeric(1.)));
//
//    // multiplication & division
//    eq!("2 * 5 / 2 * 5 / 2 * 5", Ok(Numeric(62.5)));
//
//    // mixed
//    eq!("2 + 10 / 2 - 2 * 1 + 1", Ok(Numeric(6.)));
//    eq!("10 * (2 + 1)", Ok(Numeric(30.)));
//    eq!("10 * (2 * (2 + 1) - 1) - 1", Ok(Numeric(49.)));
//
//    // TODO:
//    //eq!("10 * [2 + 1]", Ok(Numeric(30.0)));
//    //eq!("10 * [2*(2 + 1) - 1] - 1", Ok(Numeric(49.0)));
//    // reducing prefixes
//    //eq!("--1", Ok(Numeric(1.0)));
//    // multiplication without parens
//    //eq!("3*-1", Ok(Numeric(-3.0)));
//    //eq!("-(1+2)", Ok(Numeric(-3.0)));
//}
//
//#[test]
//fn parse_errors() {
//    // two numbers
//    assert!(
//        parse_str("10 10").is_err(),
//        "two numbers not allowed without operator"
//    );
//
//    // two operators
//    assert!(
//        parse_str("* /").is_err(),
//        "two operators not allowed without numbers"
//    );
//
//    // paren nesting incorrect
//    assert!(
//        parse_str("10*((2*(2+1)-1)-1").is_err(),
//        "nesting is not valid"
//    );
//    assert!(
//        parse_str("10*[(2*(2+1)-1]]-1").is_err(),
//        "nesting is not valid"
//    );
//    assert!(
//        parse_str("10*{(2*(2+1)-1)-1").is_err(),
//        "nesting is not valid"
//    );
//
//    // empty expression
//    assert!(parse_str("[]").is_err(), "empty expression is an error");
//    assert!(parse_str("(())").is_err(), "empty expression is an error");
//
//    // assignments
//    err!("2 = 10", "assignment to constant not allowed");
//
//    /*
//     * TODO: these won't be errors in near future
//    assert!(
//        exec_str_pre_num("f(2)=1").is_err(),
//        "only identifiers allowed in function assignment position"
//    );
//
//    assert!(
//        exec_str_pre_num("f(y,2)=1").is_err(),
//        "only identifiers allowed in function assignment position"
//    );
//
//    assert!(
//        exec_str_pre_num("f(2)=x(1)").is_err(),
//        "only identifiers allowed in function assignment position"
//    );
//    */
//
//    // function calls
//    err!("unknown()", "invalid call");
//    err!("sqrt()", "invalid call");
//    err!("sqrt(16, 16)", "invalid call");
//
//    // valid identifiers
//    assert!(parse_str("x").is_ok(), "invalid identifier");
//    assert!(parse_str("test").is_ok(), "invalid identifier");
//    assert!(parse_str("test1").is_ok(), "invalid identifier");
//    assert!(parse_str("1test").is_err(), "invalid identifier");
//    assert!(parse_str("f'").is_ok(), "invalid identifier");
//    assert!(parse_str("10").is_ok(), "number is treated as identifier");
//    assert!(parse_str("empty?").is_ok(), "invalid identifier");
//    assert!(parse_str("empt?y").is_err(), "invalid identifier");
//}
//
//#[test]
//fn test_stdlib() {
//    let mut repl = Repl::with_stdlib();
//
//    // constants
//    eq!(repl, "pi", Ok(Numeric(std::f64::consts::PI)));
//    eq!(repl, "e", Ok(Numeric(std::f64::consts::E)));
//
//    // sqrt
//    eq!(repl, "sqrt(16)", Ok(Numeric(4.)));
//    eq!(repl, "sqrt(64)", Ok(Numeric(8.)));
//
//    // sqrtn
//    //eq!(repl, "sqrt(64, 3)", Ok(Numeric(4.)));
//    //eq!(repl, "sqrt(3125, 5)", Ok(Numeric(5.)));
//
//    // log
//    eq!(repl, "log(8, 2)", Ok(Numeric(3.)));
//    eq!(repl, "log(100, 10)", Ok(Numeric(2.)));
//    eq!(repl, "log(100)", Ok(Numeric(2.)));
//
//    // ln
//    //eq!(repl, "ln(10)", Ok(Numeric(2.302585092994046)));
//    //eq!(repl, "ln(1)", Ok(Numeric(0.0)));
//    //eq!(repl, "ln(e)", Ok(Numeric(1.0)));
//
//    // if
//    eq!(repl, "if(1==1,1,2)", Ok(Numeric(1.)));
//    eq!(repl, "if(1!=1,1,2)", Ok(Numeric(2.)));
//
//    // assert
//    eq!(repl, "assert(1==1)", Ok(Nil));
//    eq!(repl, "assert(true)", Ok(Nil));
//
//    // empty?
//    //eq!(repl, "empty?({})", Ok(Logical(true)));
//    //eq!(repl, "empty?({1})", Ok(Logical(false)));
//}
//
//#[test]
//fn test_logical() {
//    // constants
//    eq!("true", Ok(Logical(true)));
//    eq!("false", Ok(Logical(false)));
//
//    // equal, not equal
//    eq!("1==1", Ok(Logical(true)));
//    eq!("1!=1", Ok(Logical(false)));
//
//    // ordering
//    eq!("20>10", Ok(Logical(true)));
//    eq!("10<10", Ok(Logical(false)));
//    eq!("10<1020", Ok(Logical(true)));
//    eq!("10>1020", Ok(Logical(false)));
//
//    eq!("10<=10", Ok(Logical(true)));
//    eq!("10<=11", Ok(Logical(true)));
//    eq!("10>=5", Ok(Logical(true)));
//    eq!("10>=10", Ok(Logical(true)));
//
//    // or
//    eq!("1==1 || 2==2", Ok(Logical(true)));
//    eq!("1==1 || 2!=2", Ok(Logical(true)));
//    eq!("1!=1 || 2==2", Ok(Logical(true)));
//    eq!("1!=1 || 2!=2", Ok(Logical(false)));
//
//    // and
//    eq!("1==1 && 2==2", Ok(Logical(true)));
//    eq!("1==1 && 2!=2", Ok(Logical(false)));
//    eq!("1!=1 && 2==2", Ok(Logical(false)));
//    eq!("1!=1 && 2!=2", Ok(Logical(false)));
//}
//
//#[test]
//fn context_example() {
//    let mut repl = Repl::with_stdlib();
//    eq!(repl, "even(n) = n % 2 == 0", Ok(Nil));
//    eq!(repl, "even(1)", Ok(Logical(false)));
//    eq!(repl, "even(2)", Ok(Logical(true)));
//    eq!(repl, "odd(n) = even(n) == (1 != 1)", Ok(Nil));
//    eq!(repl, "odd(1)", Ok(Logical(true)));
//    eq!(repl, "odd(2)", Ok(Logical(false)));
//
//    eq!(repl, "my_const = 10", Ok(Nil));
//    eq!(repl, "x = 2", Ok(Nil));
//    eq!(repl, "f(x) = my_const ^ x", Ok(Nil));
//    eq!(repl, "f(1)", Ok(Numeric(10.)));
//    eq!(repl, "f(x)", Ok(Numeric(100.)));
//    eq!(repl, "my_const = 2", Ok(Nil));
//    eq!(repl, "f(1)", Ok(Numeric(2.)));
//    eq!(repl, "f(x)", Ok(Numeric(4.)));
//}
//
//#[test]
//fn context_recursion() {
//    let mut repl = Repl::with_stdlib();
//    eq!(repl, "f(0) = 0", Ok(Nil));
//    eq!(repl, "f(1) = 1", Ok(Nil));
//    eq!(repl, "f(x) = f(x - 1) + f(x - 2)", Ok(Nil));
//
//    eq!(repl, "f(0)", Ok(Numeric(0.)));
//    eq!(repl, "f(1)", Ok(Numeric(1.)));
//    eq!(repl, "f(2)", Ok(Numeric(1.)));
//    eq!(repl, "f(3)", Ok(Numeric(2.)));
//    eq!(repl, "f(4)", Ok(Numeric(3.)));
//    eq!(repl, "f(5)", Ok(Numeric(5.)));
//    eq!(repl, "f(9)", Ok(Numeric(34.)));
//}
//
//#[test]
//fn test_optimize() {
//    use crate::ast::Expr::*;
//    let repl = Repl::with_stdlib();
//    let parse_optimized = |repl: &Repl, s: &str| -> Option<Expr> {
//        match ExprParser::new().parse(s) {
//            Ok(mut program) => {
//                repl.optimize(&mut program).unwrap();
//                Some(program)
//            }
//            _ => unimplemented!(),
//        }
//    };
//
//    match parse_optimized(&repl, "1 + 2 + 3") {
//        Some(Value(Numeric(n))) if n == 6. => {}
//        _ => panic!("optimization wrong"),
//    }
//
//    match parse_optimized(&repl, "1 != 1 && 1 == 1") {
//        Some(Value(Logical(false))) => {}
//        _ => panic!("optimization wrong"),
//    }
//
//    match parse_optimized(&repl, "1 == 1 || 1 != 1") {
//        Some(Value(Logical(true))) => {}
//        _ => panic!("optimization wrong"),
//    }
//
//    match parse_optimized(&repl, "x * 0") {
//        Some(Value(Numeric(n))) if n == 0. => {}
//        _ => panic!("optimization wrong"),
//    }
//}
//
//#[test]
//fn test_tuple() {
//    eq!(
//        "{1, 2, 3}",
//        Ok(Value::Set(vec![
//            Expr::Value(Value::Numeric(1.)),
//            Expr::Value(Value::Numeric(2.)),
//            Expr::Value(Value::Numeric(3.)),
//        ]))
//    );
//
//    /*
//    TODO: requires `log` to be a constant function
//    eq!(
//        "{log(2, 4), 2}",
//        Ok(Value::Set(vec![
//            Expr::Value(Value::Numeric(2.)),
//            Expr::Value(Value::Numeric(2.)),
//        ]))
//    );
//    */
//
//    eq!("{}", Ok(Value::Set(vec![])));
//
//    /*
//                // indexing
//                eq!(exec_str("{1,2,3}_2"), 3.0);
//                eq!(exec_str("{1,2,3}_2^2"), 9.0);
//                assert!(
//                    exec_str_pre_set("{1,2,3}_(1==2)^2").is_err(),
//                    "bool is not a valid index"
//                );
//
//                // generator
//                eq!(exec_str_set("{x | 0 < x, x < 5}"), vec!["1", "2", "3", "4"]);
//    */
//}
