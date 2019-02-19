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

    pub fn run(&mut self, expr: &Expr) -> VmResult
    {
        run_with_ctx(expr, &mut self.ctx)
    }
}

pub fn run_with_ctx(expr: &Expr, ctx: &mut VmContext) -> VmResult
{
    match expr {
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
                Operator::Div => arg1 / arg2,
                Operator::Pow => arg1.powf(arg2),
                Operator::Mod => arg1 % arg2,
                _ => unreachable!(),
            };

            Ok(result.into())
        }
        Expr::Value(v) => Ok(v.clone()),
        Expr::Ref(name) => run_lookup(name, ctx),
        Expr::Func(name, params) => run_function(name, params, ctx),
    }
}

pub fn run_lookup(name: &RefType, ctx: &mut VmContext) -> VmResult
{
    if let Some(entry) = ctx.get(name) {
        // TODO: execute all params before pushing them into a function
        //let params = params.iter().map(|e| self.run_with_ctx(e,
        // ctx).unwrap()).collect();
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
    if let Some(entry) = ctx.get(name) {
        // TODO: execute all params before pushing them into a function
        //let params = params.iter().map(|e| self.run_with_ctx(e,
        // ctx).unwrap()).collect();
        match entry {
            // TODO: find a way around that clone
            VmFunction::Virtual((_args, n)) => run_with_ctx(&n.clone(), ctx),
            VmFunction::Native(func) => func(params, ctx),
        }
    } else {
        Err(format!("function `{}` is unknown", name))
    }
}
