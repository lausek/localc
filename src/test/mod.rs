#![cfg(test)]

mod perf;

use super::*;

#[allow(unused_imports)]
use lovm::{gen, vm, *};

#[macro_export]
macro_rules! expect {
    ($repl:expr, $line:expr, $expect:expr) => {{
        fn debug(data: &mut vm::VmData) -> vm::VmResult {
            let result = data.vstack.last_mut().expect("no value");
            println!("{:?}", result);
            assert_eq!(result, &$expect);
            Ok(())
        }
        $repl
            .runtime
            .vm
            .interrupts_mut()
            .set(vm::Interrupt::Debug as usize, &debug);
        $repl.run($line).unwrap();
        $repl
            .runtime
            .vm
            .interrupts_mut()
            .unset(vm::Interrupt::Debug as usize);
    }};
}

#[test]
fn fib() {
    let mut repl = Repl::new();

    repl.run("f(0) = 0").unwrap();
    repl.run("f(1) = 1").unwrap();
    repl.run("f(x) = f(x - 1) + f(x - 2)").unwrap();

    expect!(repl, "f(0)", lovm::Value::I64(0));
    expect!(repl, "f(1)", lovm::Value::I64(1));
    expect!(repl, "f(8)", lovm::Value::I64(21));
}

#[test]
fn sqrt() {
    let mut repl = Repl::new();

    repl.run("sqrt(x) = 1.0 * x ^ (1.0 / 2)").unwrap();
    repl.run("sqrt(x,n) = 1.0 * x ^ (1.0 / n)").unwrap();

    expect!(repl, "sqrt(4)", lovm::Value::F64(2.));
    expect!(repl, "sqrt(27, 3)", lovm::Value::F64(3.));
}

#[test]
fn multiargs() {
    // implements crazy logic
    // return false if (0, 1)
    // return true if (1, 0)
    // return `eq` if (x, y)

    let mut repl = Repl::new();

    repl.run("f(0, 1) = false").unwrap();
    repl.run("f(1, 0) = true").unwrap();
    repl.run("f(x, y) = x == y").unwrap();

    expect!(repl, "f(0, 1)", lovm::Value::T(false));
    expect!(repl, "f(1, 0)", lovm::Value::T(true));
    expect!(repl, "f(1, 1)", lovm::Value::T(true));
}

#[test]
fn numeric() {
    let mut repl = Repl::new();

    // addition
    expect!(repl, "1 + 1", lovm::Value::I64(2));
    expect!(repl, "18 + 18", lovm::Value::I64(36));

    // subtraction
    expect!(repl, "18 - 18", lovm::Value::I64(0));

    // multiplication
    expect!(repl, "18 * 18", lovm::Value::I64(324));

    // division
    expect!(repl, "18 / 18", lovm::Value::I64(1));

    // power
    expect!(repl, "10 ^ 3", lovm::Value::I64(1000));

    // modulo
    expect!(repl, "8 % 2", lovm::Value::I64(0));
    expect!(repl, "9 % 2", lovm::Value::I64(1));

    // division with zero
    //err!("18 / 0", "division by zero not allowed");

    // different priority
    expect!(repl, "1 + 4 * 5", lovm::Value::I64(21));
    expect!(repl, "(1 + 4) * 5", lovm::Value::I64(25));
    expect!(repl, "(1.0 + 4) * 5 / 2", lovm::Value::F64(12.5));

    // addition & subtraction
    expect!(
        repl,
        "1 + 1 - 1 + 1 - 1 + 1 - 1 + 1 - 1",
        lovm::Value::I64(1)
    );

    // multiplication & division
    expect!(repl, "2.0 * 5 / 2 * 5 / 2 * 5", lovm::Value::F64(62.5));

    // mixed
    expect!(repl, "2.0 + 10 / 2 - 2 * 1 + 1", lovm::Value::F64(6.));
    expect!(repl, "10.0 * (2 + 1)", lovm::Value::F64(30.));
    expect!(repl, "10.0 * (2 * (2 + 1) - 1) - 1", lovm::Value::F64(49.));

    // TODO: multiplication without parens
    //eq!("3*-1", Ok(Numeric(-3.0)));
    //eq!("-(1+2)", Ok(Numeric(-3.0)));
}

#[test]
fn logical() {
    let mut repl = Repl::new();

    // constants
    expect!(repl, "true", lovm::Value::T(true));
    expect!(repl, "false", lovm::Value::T(false));

    // equal, not equal
    expect!(repl, "1==1", lovm::Value::T(true));
    expect!(repl, "1!=1", lovm::Value::T(false));

    // ordering
    expect!(repl, "20>10", lovm::Value::T(true));
    expect!(repl, "10<10", lovm::Value::T(false));
    expect!(repl, "10<1020", lovm::Value::T(true));
    expect!(repl, "10>1020", lovm::Value::T(false));

    expect!(repl, "10<=10", lovm::Value::T(true));
    expect!(repl, "10<=11", lovm::Value::T(true));
    expect!(repl, "10>=5", lovm::Value::T(true));
    expect!(repl, "10>=10", lovm::Value::T(true));

    // or
    expect!(repl, "1==1 || 2==2", lovm::Value::T(true));
    expect!(repl, "1==1 || 2!=2", lovm::Value::T(true));
    expect!(repl, "1!=1 || 2==2", lovm::Value::T(true));
    expect!(repl, "1!=1 || 2!=2", lovm::Value::T(false));

    // and
    expect!(repl, "1==1 && 2==2", lovm::Value::T(true));
    expect!(repl, "1==1 && 2!=2", lovm::Value::T(false));
    expect!(repl, "1!=1 && 2==2", lovm::Value::T(false));
    expect!(repl, "1!=1 && 2!=2", lovm::Value::T(false));
}

#[test]
fn context_example() {
    let mut repl = Repl::new();

    repl.run("even(n) = n % 2 == 0").unwrap();
    repl.run("odd(n) = even(n) == (1 != 1)").unwrap();
    repl.run("f(x) = my_const ^ x").unwrap();
    repl.run("my_const = 10").unwrap();
    repl.run("x = 2").unwrap();

    expect!(repl, "even(1)", lovm::Value::T(false));
    expect!(repl, "even(2)", lovm::Value::T(true));

    expect!(repl, "odd(1)", lovm::Value::T(true));
    expect!(repl, "odd(2)", lovm::Value::T(false));

    expect!(repl, "f(1)", lovm::Value::I64(10));
    expect!(repl, "f(x)", lovm::Value::I64(100));

    repl.run("my_const = 2").unwrap();
    expect!(repl, "f(1)", lovm::Value::I64(2));
    expect!(repl, "f(x)", lovm::Value::I64(4));
}

#[test]
fn tuples() {
    use lovm::vm::object::*;

    let mut repl = Repl::new();

    repl.run("f(n) = n ^ 2").unwrap();
    expect!(repl, "(1,2,3,f(2))", lovm::Value::Ref(1));

    let tuple = repl.runtime.vm.data.obj_pool.get(&1).unwrap();
    let cmp_obj = ObjectKind::Array(
        vec![
            lovm::Value::I64(1),
            lovm::Value::I64(2),
            lovm::Value::I64(3),
            lovm::Value::I64(4),
        ]
        .into(),
    );

    assert!(tuple.inner == cmp_obj);
}

#[test]
fn sets() {
    use lovm::vm::object::*;

    let mut repl = Repl::new();

    // TODO: won't work because string parser is still a bit stupid
    expect!(repl, "{ 5 = 10, 1.0, 4 = 20 }", lovm::Value::Ref(1));

    let set = repl.runtime.vm.data.obj_pool.get(&1).unwrap();

    match &set.inner {
        ObjectKind::Dict(got) => {
            let got = got.inner();
            assert_eq!(got[&lovm::Value::I64(5)], lovm::Value::I64(10));
            assert_eq!(got[&lovm::Value::I64(4)], lovm::Value::I64(20));
            assert_eq!(got[&lovm::Value::I64(1)], lovm::Value::F64(1.));
        }
        _ => unreachable!(),
    }
}

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
