use super::*;

use lovm::gen::*;
use lovm::*;

pub type CompileResult = Result<CodeObject, String>;

pub fn compile_files(files: &Vec<String>) -> Result<lovm::Module, String> {
    use std::fs::File;
    use std::io::Read;

    let mut parser = ExprParser::new();
    let mut module = ModuleBuilder::new();

    for file in files.iter() {
        match File::open(file) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                for line in content.lines().filter(|line| !line.is_empty()) {
                    let ast = parser.parse(line).expect("parse error");
                    //match ast {
                    //    Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
                    //        box Expr::Func(name, params) => self.store_fun(name, params, rhs),
                    //        box Expr::Ref(name) => self.store_var(name, rhs),
                    //        _ => Err("assignment not allowed".to_string()),
                    //    },
                    //    _ => {
                    //        // TODO: if this returns a reference to an temporary object; drop it to save memory
                    //        // no storage location given: execute directly
                    //        self.run(&compiler::compile_expr(expr)?)
                    //    }
                    //}
                }
            }
            _ => return Err(format!("could not open file `{}`", file)),
        }
    }

    Ok(module.build().unwrap())
}

pub fn compile_str(s: &str) -> CompileResult {
    let expr = ExprParser::new().parse(s.as_ref()).unwrap();
    compile_expr(&expr)
}

pub fn compile_with_params_lazy(ast: &Expr, params: &TupleType) -> Result<FunctionBuilder, String> {
    let params = params
        .iter()
        .filter_map(|param| match param {
            Expr::Ref(n) => Some(n.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut func = FunctionBuilder::new().with_params(params);
    let mut op_stack = vec![];
    compile_deep(&mut func, &mut op_stack, ast)?;
    Ok(func)
}

pub fn compile_with_params(ast: &Expr, params: &TupleType) -> CompileResult {
    let func = compile_with_params_lazy(ast, params)?;
    let func = func.build().unwrap();
    Ok(func.into())
}

pub fn compile_expr(ast: &Expr) -> CompileResult {
    let mut func = FunctionBuilder::new();
    let mut op_stack = vec![];
    compile_deep(&mut func, &mut op_stack, ast)?;
    func.debug();
    let func = func.build().unwrap();
    Ok(func.into())
}

fn compile_deep(
    func: &mut FunctionBuilder,
    op_stack: &mut Vec<Operation>,
    ast: &Expr,
) -> Result<(), String> {
    match ast {
        // store must be handled before compilation to support dynamic function dispatch.
        Expr::Comp(Operator::Store, _, _) => unimplemented!(),
        Expr::Comp(op, lhs, rhs) => {
            op_stack.push(Operation::new(op.into()));
            compile_deep(func, op_stack, &lhs)?;
            compile_deep(func, op_stack, &rhs)?;
            let op = op_stack.pop().unwrap();
            if let Some(last) = op_stack.last_mut() {
                last.op(op);
            } else {
                func.step(op.end());
            }
        }
        Expr::Func(name, params) => {
            op_stack.push(Operation::call(name));
            // calling order does not need `.rev()`
            for param in params.iter() {
                compile_deep(func, op_stack, &param)?;
            }
            // pass argc to function call
            op_stack.last_mut().unwrap().op(params.len());

            let op = op_stack.pop().unwrap();
            if let Some(last) = op_stack.last_mut() {
                last.op(op);
            } else {
                func.step(op.end());
            }
        }
        Expr::Ref(r) => {
            if let Some(last) = op_stack.last_mut() {
                last.var(r.as_ref());
            } else {
                func.step(Operation::push().var(r.clone()).end());
            }
        }
        Expr::Value(crate::ast::Value::Numeric(NumType::Natural(v))) => {
            if let Some(last) = op_stack.last_mut() {
                last.op(*v);
            } else {
                func.step(Operation::push().op(*v).end());
            }
        }
        Expr::Value(crate::ast::Value::Numeric(NumType::Rational(v))) => {
            if let Some(last) = op_stack.last_mut() {
                last.op(*v);
            } else {
                func.step(Operation::push().op(*v).end());
            }
        }
        Expr::Value(crate::ast::Value::Logical(v)) => {
            if let Some(last) = op_stack.last_mut() {
                last.op(*v);
            } else {
                func.step(Operation::push().op(*v).end());
            }
        }
        Expr::Value(crate::ast::Value::Str(v)) => {
            if let Some(last) = op_stack.last_mut() {
                last.op(v.as_ref());
            } else {
                func.step(Operation::push().op(v.as_ref()).end());
            }
        }
        Expr::Value(crate::ast::Value::Tuple(tuple)) => {
            // TODO: this could cause stack indexing problems
            if let Some(last) = op_stack.last_mut() {
                last.op(tuple.clone());
            } else {
                func.step(Operation::push().op(tuple.clone()).end());
            }
        }
        Expr::Value(crate::ast::Value::Set(set)) => {
            // TODO: this could cause stack indexing problems
            if let Some(last) = op_stack.last_mut() {
                last.op(set.clone());
            } else {
                func.step(Operation::push().op(set.clone()).end());
            }
        }
        other => println!("`{:?}` not yet implemented", other),
    }
    Ok(())
}

impl From<&Operator> for OperationType {
    fn from(from: &Operator) -> Self {
        match from {
            Operator::Add => OperationType::Add,
            Operator::Sub => OperationType::Sub,
            Operator::Mul => OperationType::Mul,
            Operator::Div => OperationType::Div,
            Operator::Pow => OperationType::Pow,
            Operator::Rem => OperationType::Rem,

            Operator::Eq => OperationType::CmpEq,
            Operator::Ne => OperationType::CmpNe,
            Operator::Ge => OperationType::CmpGe,
            Operator::Gt => OperationType::CmpGt,
            Operator::Le => OperationType::CmpLe,
            Operator::Lt => OperationType::CmpLt,

            Operator::And => OperationType::And,
            Operator::Or => OperationType::Or,
            _ => panic!("from not implemented for `{:?}`", from),
        }
    }
}
