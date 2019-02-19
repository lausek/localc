pub mod context;

use self::context::*;
use crate::ast::*;

pub type VmValue = Value;
pub type VmError = String;
pub type VmResult = Result<VmValue, VmError>;

pub struct Vm {}

impl Vm
{
    pub fn new() -> Self
    {
        Self {}
    }

    pub fn run(&mut self, expr: &Expr) -> VmResult
    {
        let mut ctx = VmContext::new();
        self.run_with_ctx(expr, &mut ctx)
    }

    pub fn run_with_ctx(&mut self, expr: &Expr, ctx: &mut VmContext) -> VmResult
    {
        match expr {
            Expr::Comp(op, arg1, arg2) => {
                let arg1 = self.run_with_ctx(arg1, ctx)?;
                let arg2 = self.run_with_ctx(arg2, ctx)?;

                let arg1 = NumType::from(arg1);
                let arg2 = NumType::from(arg2);

                let result = match op {
                    Operator::Add => arg1 + arg2,
                    Operator::Sub => arg1 - arg2,
                    Operator::Mul => arg1 * arg2,
                    Operator::Div => arg1 / arg2,
                    Operator::Pow => arg1.powf(arg2),
                    Operator::Mod => arg1 % arg2,
                };

                Ok(result.into())
            }
            Expr::Value(v) => Ok(v.clone()),
            Expr::Ref(name) => self.run_lookup(name, ctx),
            Expr::Func(name, params) => self.run_function(name, params, ctx),
        }
    }

    pub fn run_lookup(&mut self, name: &RefType, ctx: &mut VmContext) -> VmResult
    {
        if let Some(entry) = ctx.get(name) {
            // TODO: execute all params before pushing them into a function
            //let params = params.iter().map(|e| self.run_with_ctx(e,
            // ctx).unwrap()).collect();
            match entry {
                // TODO: find a way around that clone
                VmFunction::Virtual(n) => self.run_with_ctx(&n.clone(), ctx),
                // TODO: there must be an easier way to specify empty params. Option<Vec<>> maybe?
                VmFunction::Native(func) => func(&vec![], ctx),
            }
        } else {
            Err(format!("variable `{}` is unknown", name))
        }
    }

    pub fn run_function(
        &mut self,
        name: &RefType,
        params: &TupleType,
        ctx: &mut VmContext,
    ) -> VmResult
    {
        if let Some(entry) = ctx.get(name) {
            // TODO: execute all params before pushing them into a function
            //let params = params.iter().map(|e| self.run_with_ctx(e,
            // ctx).unwrap()).collect();
            match entry {
                // TODO: find a way around that clone
                VmFunction::Virtual(n) => self.run_with_ctx(&n.clone(), ctx),
                VmFunction::Native(func) => func(params, ctx),
            }
        } else {
            Err(format!("function `{}` is unknown", name))
        }
    }
}
