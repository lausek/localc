use super::{Expr::*, Value::*, *};

use std::collections::HashMap;

pub type VmContextEntry = VmFunction;
pub type VmFunctionVirtual = (TupleType, Box<Expr>);
pub type VmFunctionNative = fn(&TupleType, &mut VmContext) -> VmResult;

#[derive(Clone)]
pub enum VmFunction
{
    Virtual(VmFunctionVirtual),
    Native(VmFunctionNative),
}

impl std::fmt::Debug for VmFunction
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            VmFunction::Virtual(n) => write!(f, "{:?}", n),
            VmFunction::Native(_) => write!(f, "<native>"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmContext
{
    map: HashMap<RefType, VmContextEntry>,
}

impl VmContext
{
    pub fn new() -> Self
    {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn stdlib() -> Self
    {
        let mut ctx = Self::new();

        ctx.map
            .insert(String::from("print"), VmFunction::Native(vm_func_print));
        ctx.map
            .insert(String::from("pi"), VmFunction::Native(vm_func_pi));
        ctx.map
            .insert(String::from("sqrt"), VmFunction::Native(vm_func_sqrt));
        ctx.map
            .insert(String::from("log"), VmFunction::Native(vm_func_log));

        ctx
    }

    pub fn get(&self, name: &RefType) -> Option<&VmContextEntry>
    {
        self.map.get(name)
    }

    pub fn set(&mut self, name: &RefType, entry: VmContextEntry)
    {
        self.map.insert(name.clone(), entry);
    }
}

fn vm_func_print(params: &TupleType, _ctx: &mut VmContext) -> VmResult
{
    for param in params {
        print!("{:?}", param);
    }
    Ok(Numeric(0.0))
}

fn vm_func_pi(_params: &TupleType, _ctx: &mut VmContext) -> VmResult
{
    Ok(Numeric(std::f64::consts::PI))
}

fn vm_func_sqrt(params: &TupleType, ctx: &mut VmContext) -> VmResult
{
    let params = run_tuple_exprs(params, ctx)?;
    let mut params = params.iter();
    match (params.next(), params.next()) {
        (Some(Value(Numeric(from))), None) => Ok(Numeric(from.sqrt())),
        (Some(Value(Numeric(from))), Some(Value(Numeric(n)))) => Ok(Numeric(from.powf(1. / n))),
        (_, _) => Err("function `sqrt` expected some paramaters".to_string()),
    }
}

fn vm_func_log(params: &TupleType, ctx: &mut VmContext) -> VmResult
{
    let params = run_tuple_exprs(params, ctx)?;
    let mut params = params.iter();
    match (params.next(), params.next()) {
        (Some(Value(Numeric(from))), None) => Ok(Numeric(from.log10())),
        (Some(Value(Numeric(from))), Some(Value(Numeric(n)))) => Ok(Numeric(from.log(*n))),
        (_, _) => Err("function `log` expected some paramaters".to_string()),
    }
}
