pub mod context;

use self::context::*;
use crate::{ast::*, expr::ExprParser};

pub type VmValue = Value;
pub type VmError = String;
pub type VmResult = Result<VmValue, VmError>;

impl std::cmp::PartialEq for Value
{
    fn eq(&self, rhs: &Self) -> bool
    {
        match (self, rhs) {
            (Value::Numeric(lhs), Value::Numeric(rhs)) => lhs == rhs,
            (Value::Logical(lhs), Value::Logical(rhs)) => lhs == rhs,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl std::cmp::PartialOrd for Value
{
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering>
    {
        match (self, rhs) {
            (Value::Numeric(lhs), Value::Numeric(rhs)) => lhs.partial_cmp(rhs),
            _ => unimplemented!(),
        }
    }
}

pub struct Vm
{
    pub parser: ExprParser,
    ctx: VmContext,
}

impl Vm
{
    pub fn new() -> Self
    {
        Self {
            parser: ExprParser::new(),
            ctx: VmContext::new(),
        }
    }

    pub fn with_stdlib() -> Self
    {
        Self {
            parser: ExprParser::new(),
            ctx: VmContext::stdlib(),
        }
    }

    pub fn run_raw(&mut self, raw: &str) -> VmResult
    {
        let program = self.parser.parse(raw).unwrap();
        run_with_ctx(&program, &mut self.ctx)
    }

    pub fn run(&mut self, expr: &Expr) -> VmResult
    {
        run_with_ctx(expr, &mut self.ctx)
    }

    pub fn optimize(&self, expr: &mut Expr) -> Result<(), String>
    {
        optimize(expr)
    }
}

fn optimize(expr: &mut Expr) -> Result<(), String>
{
    let mut new_val = None;
    match expr {
        Expr::Comp(op, box Expr::Value(lhs), box Expr::Value(rhs)) => {
            new_val = Some(run_operation(&op, &lhs, &rhs));
        }
        Expr::Comp(op, lhs, rhs) => {
            // optimize lazily because `and`, `or` must not evaluate the second operand
            // if the first one already has a known value
            optimize(lhs).unwrap();
            match (&op, &lhs) {
                (Operator::And, box Expr::Value(Value::Logical(false))) => {
                    new_val = Some(Ok(Value::Logical(false)));
                }
                (Operator::Or, box Expr::Value(Value::Logical(true))) => {
                    new_val = Some(Ok(Value::Logical(true)));
                }
                _ => {
                    optimize(rhs).unwrap();
                    match (&op, &rhs) {
                        (Operator::And, box Expr::Value(Value::Logical(false))) => {
                            new_val = Some(Ok(Value::Logical(false)));
                        }
                        (Operator::Or, box Expr::Value(Value::Logical(true))) => {
                            new_val = Some(Ok(Value::Logical(true)));
                        }
                        _ => match (lhs, rhs) {
                            // multiplications with 0 turn out to 0 regardless
                            // of wether or not the second operand is constant
                            (_, box Expr::Value(Value::Numeric(n)))
                            | (box Expr::Value(Value::Numeric(n)), _)
                                if Operator::Mul == *op && *n == 0. =>
                            {
                                new_val = Some(Ok(Value::Numeric(0.)));
                            }
                            (box Expr::Value(lhs), box Expr::Value(rhs)) => {
                                new_val = Some(run_operation(&op, &lhs, &rhs));
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
        Expr::Func(_m, params) => {
            for param in params {
                optimize(param).unwrap();
            }
        }
        _ => {}
    }
    match new_val {
        Some(Ok(val)) => {
            info!("optimizing expr: {:?}", expr);
            info!("replacing with const value: {:?}", val);
            *expr = Expr::Value(val);
        }
        Some(_) => panic!("error in constant optimization"),
        _ => {}
    }
    Ok(())
}

pub fn run_with_ctx(expr: &Expr, ctx: &mut VmContext) -> VmResult
{
    match expr {
        Expr::Value(v) => Ok(v.clone()),
        Expr::Ref(name) => run_lookup(name, ctx),
        Expr::Func(name, params) => run_function(name, params, ctx),
        Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
            box Expr::Func(name, params) => {
                ctx.set_virtual(name, (params.clone(), rhs.clone()));
                Ok(Value::Nil)
            }
            box Expr::Ref(name) => {
                ctx.set_virtual(name, (vec![], rhs.clone()));
                Ok(Value::Nil)
            }
            _ => Err(format!("`{:?}` is not assignable", lhs)),
        },
        Expr::Comp(op, arg1, arg2) => {
            let arg1 = run_with_ctx(arg1, ctx)?;
            let arg2 = run_with_ctx(arg2, ctx)?;
            run_operation(op, &arg1, &arg2)
        }
    }
}

pub fn run_operation(op: &Operator, arg1: &Value, arg2: &Value) -> VmResult
{
    match op {
        Operator::Add
        | Operator::Sub
        | Operator::Mul
        | Operator::Div
        | Operator::Rem
        | Operator::Pow => exec_num_op(&op, &arg1, &arg2),
        Operator::Eq | Operator::Ne | Operator::Ge | Operator::Gt | Operator::Le | Operator::Lt => {
            exec_value_op(&op, &arg1, &arg2)
        }
        Operator::And | Operator::Or => exec_log_op(&op, &arg1, &arg2),
        _ => unimplemented!(),
    }
}

pub fn run_lookup(name: &RefType, ctx: &mut VmContext) -> VmResult
{
    info!("lookup: {}", name);
    if let Some(entry) = ctx.get(name) {
        match &*(entry.borrow()) {
            VmFunction::Virtual(table) => {
                let (_args, expr) = lookup_func(table, &vec![]).unwrap();
                info!("resulted in: {:?}", expr);
                run_with_ctx(expr, ctx)
            }
            // TODO: there must be an easier way to specify empty params. Option<Vec<>> maybe?
            VmFunction::Native(func) => func(&vec![], ctx),
        }
    } else {
        Err(format!("function `{}` is unknown", name))
    }
}

pub fn run_function(name: &RefType, params: &TupleType, ctx: &mut VmContext) -> VmResult
{
    info!("function: {} ({:?})", name, params);
    if let Some(entry) = ctx.get(name) {
        match &*(entry.borrow()) {
            VmFunction::Virtual(table) => {
                let params = run_tuple_exprs(params, ctx)?;
                match lookup_func(table, &params) {
                    Some((args, expr)) => {
                        info!("resulted in: {:?}", expr);
                        push_ctx_params(ctx, &args, &params);
                        let result = run_with_ctx(&expr, ctx);
                        pop_ctx_params(ctx);
                        result
                    }
                    _ => Err("unexpected arguments".to_string()),
                }
            }
            VmFunction::Native(func) => func(&params, ctx),
        }
    } else {
        Err(format!("function `{}` is unknown", name))
    }
}

fn run_tuple_exprs(params: &TupleType, ctx: &mut VmContext) -> Result<TupleType, VmError>
{
    let mut list = vec![];
    for param in params {
        let result = run_with_ctx(&param, ctx)?;
        list.push(Expr::Value(result));
    }
    Ok(list)
}

fn push_ctx_params(ctx: &mut VmContext, names: &TupleType, vals: &TupleType)
{
    use std::cell::RefCell;
    use std::rc::Rc;
    // TODO: probably not needed
    assert_eq!(names.len(), vals.len());
    let frame = names
        .iter()
        .zip(vals)
        .filter_map(|(name, val)| {
            let expr = Box::new(val.clone());
            match name {
                Expr::Ref(name) => {
                    let table = vec![(vec![], expr)];
                    let func = VmFunction::Virtual(table);
                    Some((name.clone(), Rc::new(RefCell::new(func))))
                }
                Expr::Value(_) => None,
                _ => unimplemented!(),
            }
        })
        .collect::<VmFrame>();
    ctx.push_frame(frame);
}

fn pop_ctx_params(ctx: &mut VmContext)
{
    assert!(ctx.pop_frame());
}

#[inline]
fn exec_num_op(op: &Operator, arg1: &Value, arg2: &Value) -> VmResult
{
    let lhs = NumType::from(arg1);
    let rhs = NumType::from(arg2);

    Ok(match op {
        Operator::Add => lhs + rhs,
        Operator::Sub => lhs - rhs,
        Operator::Mul => lhs * rhs,
        Operator::Div => {
            if rhs != 0. {
                lhs / rhs
            } else {
                return Err("division with 0".to_string());
            }
        }
        Operator::Pow => lhs.powf(rhs),
        Operator::Rem => lhs % rhs,
        _ => unimplemented!(),
    }
    .into())
}

#[inline]
fn exec_log_op(op: &Operator, arg1: &Value, arg2: &Value) -> VmResult
{
    let lhs = LogType::from(arg1);
    let rhs = LogType::from(arg2);

    Ok(match op {
        Operator::And => lhs && rhs,
        Operator::Or => lhs || rhs,
        _ => unimplemented!(),
    }
    .into())
}

#[inline]
fn exec_value_op(op: &Operator, arg1: &Value, arg2: &Value) -> VmResult
{
    Ok(match op {
        Operator::Eq => arg1 == arg2,
        Operator::Ne => arg1 != arg2,
        Operator::Ge => arg1 >= arg2,
        Operator::Gt => arg1 > arg2,
        Operator::Le => arg1 <= arg2,
        Operator::Lt => arg1 < arg2,
        _ => unimplemented!(),
    }
    .into())
}
