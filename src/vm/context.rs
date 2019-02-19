use super::*;

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
        let mut ctx = Self {
            map: HashMap::new(),
        };

        ctx.map
            .insert(String::from("print"), VmFunction::Native(vm_func_print));
        ctx.map
            .insert(String::from("pi"), VmFunction::Native(vm_func_get_pi));

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
    Ok(Value::Numeric(0.0))
}

fn vm_func_get_pi(_params: &TupleType, _ctx: &mut VmContext) -> VmResult
{
    Ok(Value::Numeric(std::f64::consts::PI))
}
