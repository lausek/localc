use self::VmFunction::*;
use super::{Expr::*, Value::*, *};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type VmFrame = Vec<(RefType, VmContextEntryRef)>;
pub type VmContextEntry = VmFunction;
pub type VmContextEntryRef = Rc<RefCell<VmFunction>>;
pub type VmFunctionVirtual = (TupleType, Box<Expr>);
pub type VmFunctionNative = fn(&TupleType, &mut Box<dyn Lookable>) -> VmResult;

#[derive(Clone)]
pub enum VmFunction
{
    Virtual(VmFunctionVirtual),
    Native(VmFunctionNative),
}

macro_rules! insert_func {
    ($ctx:expr, $name:expr, $func:ident) => {
        let r = VmFunction::Native($func);
        $ctx.insert($name.to_string(), Rc::new(RefCell::new(r)));
    };
    ($ctx:expr, $name:expr, $expr:expr) => {
        let r = VmFunction::Virtual($expr);
        $ctx.insert($name.to_string(), Rc::new(RefCell::new(r)));
    };
}

pub trait Lookable
{
    fn get(&self, name: &RefType) -> Option<VmContextEntryRef>;
    fn set(&mut self, name: &RefType, entry: VmContextEntry);
    fn push_frame(&mut self, frame: VmFrame);
    fn pop_frame(&mut self) -> bool;
}

impl std::fmt::Debug for VmFunction
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Virtual(n) => write!(f, "{:?}", n),
            Native(_) => write!(f, "<native>"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmContext
{
    map: HashMap<RefType, VmContextEntryRef>,
    stack: Vec<VmFrame>,
}

impl VmContext
{
    pub fn new() -> Self
    {
        Self {
            map: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn stdlib() -> Self
    {
        let mut ctx = Self::new();

        insert_func!(ctx.map, "pi", vm_func_pi);
        insert_func!(ctx.map, "print", vm_func_print);
        insert_func!(ctx.map, "sqrt", vm_func_sqrt);
        insert_func!(ctx.map, "log", vm_func_log);

        ctx
    }
}

impl Lookable for VmContext
{
    fn push_frame(&mut self, frame: VmFrame)
    {
        self.stack.push(frame);
    }

    fn pop_frame(&mut self) -> bool
    {
        self.stack.pop();
        true
    }

    fn get(&self, name: &RefType) -> Option<VmContextEntryRef>
    {
        for frame in self.stack.iter().rev() {
            for (var, val) in frame.iter() {
                if var == name {
                    println!("found {:?} in stack", var);
                    return Some(val.clone());
                }
            }
        }
        self.map.get(name).map(|r| r.clone())
    }

    fn set(&mut self, name: &RefType, entry: VmContextEntry)
    {
        self.map.insert(name.clone(), Rc::new(RefCell::new(entry)));
    }
}

fn vm_func_print(params: &TupleType, _ctx: &mut Box<dyn Lookable>) -> VmResult
{
    for param in params {
        print!("{:?}", param);
    }
    Ok(Numeric(0.0))
}

fn vm_func_pi(_params: &TupleType, _ctx: &mut Box<dyn Lookable>) -> VmResult
{
    Ok(Numeric(std::f64::consts::PI))
}

fn vm_func_sqrt(params: &TupleType, ctx: &mut Box<dyn Lookable>) -> VmResult
{
    let params = run_tuple_exprs(params, ctx)?;
    let mut params = params.iter();
    match (params.next(), params.next()) {
        (Some(Value(Numeric(from))), None) => Ok(Numeric(from.sqrt())),
        (Some(Value(Numeric(from))), Some(Value(Numeric(n)))) => Ok(Numeric(from.powf(1. / n))),
        (_, _) => Err("function `sqrt` expected some paramaters".to_string()),
    }
}

fn vm_func_log(params: &TupleType, ctx: &mut Box<dyn Lookable>) -> VmResult
{
    let params = run_tuple_exprs(params, ctx)?;
    let mut params = params.iter();
    match (params.next(), params.next()) {
        (Some(Value(Numeric(from))), None) => Ok(Numeric(from.log10())),
        (Some(Value(Numeric(from))), Some(Value(Numeric(n)))) => Ok(Numeric(from.log(*n))),
        (_, _) => Err("function `log` expected some paramaters".to_string()),
    }
}
