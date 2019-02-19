use crate::ast::*;

use std::collections::HashMap;

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
            _ => unimplemented!(),
        }
    }
}

pub struct VmContext
{
    map: HashMap<RefType, Box<Expr>>,
}

impl VmContext
{
    pub fn new() -> Self
    {
        Self {
            map: HashMap::new(),
        }
    }
}
