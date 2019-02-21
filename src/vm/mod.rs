pub mod context;

use self::context::*;
use crate::ast::*;

pub type VmValue = Value;
pub type VmError = String;
pub type VmResult = Result<VmValue, VmError>;

pub struct Vm
{
    ctx: VmContext,
}

impl Vm
{
    pub fn new() -> Self
    {
        Self {
            ctx: VmContext::new(),
        }
    }

    pub fn with_stdlib() -> Self
    {
        Self {
            ctx: VmContext::stdlib(),
        }
    }

    pub fn run(&mut self, expr: &Expr) -> VmResult
    {
        run_with_ctx(expr, &mut self.ctx)
    }
}

pub fn run_with_ctx(expr: &Expr, ctx: &mut VmContext) -> VmResult
{
    match expr {
        Expr::Value(v) => Ok(v.clone()),
        Expr::Ref(name) => run_lookup(name, ctx),
        Expr::Func(name, params) => run_function(name, params, ctx),
        Expr::Comp(Operator::Equ, lhs, rhs) => match lhs {
            box Expr::Func(name, params) => {
                let func = VmFunction::Virtual((params.clone(), rhs.clone()));
                ctx.set(name, func);
                Ok(Value::Empty)
            }
            box Expr::Ref(name) => {
                let val = VmFunction::Virtual((vec![], rhs.clone()));
                ctx.set(name, val);
                Ok(Value::Empty)
            }
            _ => Err(format!("`{:?}` is not assignable", lhs)),
        },
        Expr::Comp(op, arg1, arg2) => {
            let arg1 = run_with_ctx(arg1, ctx)?;
            let arg2 = run_with_ctx(arg2, ctx)?;

            let arg1 = NumType::from(arg1);
            let arg2 = NumType::from(arg2);

            let result = match op {
                Operator::Add => arg1 + arg2,
                Operator::Sub => arg1 - arg2,
                Operator::Mul => arg1 * arg2,
                Operator::Div => {
                    if arg2 != 0.0 {
                        arg1 / arg2
                    } else {
                        return Err("division with 0".to_string());
                    }
                }
                Operator::Pow => arg1.powf(arg2),
                Operator::Mod => arg1 % arg2,
                _ => unreachable!(),
            };

            Ok(result.into())
        }
    }
}

pub fn run_lookup(name: &RefType, ctx: &mut VmContext) -> VmResult
{
    if let Some(entry) = ctx.get(name) {
        match entry {
            // TODO: find a way around that clone
            VmFunction::Virtual((_args, n)) => run_with_ctx(&n.clone(), ctx),
            // TODO: there must be an easier way to specify empty params. Option<Vec<>> maybe?
            VmFunction::Native(func) => func(&vec![], ctx),
        }
    } else {
        Err(format!("variable `{}` is unknown", name))
    }
}

pub fn run_function(name: &RefType, params: &TupleType, ctx: &mut VmContext) -> VmResult
{
    let params = run_tuple_exprs(params, ctx)?;
    if let Some(entry) = ctx.get(name) {
        // TODO: execute all params before pushing them into a function
        match entry {
            // TODO: find a way around that clone
            VmFunction::Virtual((args, n)) => {
                let mut local_ctx = remap_ctx_params(ctx, args, &params);
                run_with_ctx(&n.clone(), &mut local_ctx)
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

fn remap_ctx_params(orig: &VmContext, names: &TupleType, vals: &TupleType) -> VmContext
{
    let mut copy = orig.clone();
    for (name, val) in names.iter().zip(vals) {
        let expr = Box::new(val.clone());
        match name {
            Expr::Ref(name) => copy.set(name, VmFunction::Virtual((vec![], expr))),
            _ => unimplemented!(),
        }
    }
    copy
}
