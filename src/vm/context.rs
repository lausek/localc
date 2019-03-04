use self::VmFunction::*;
use super::{Expr::*, Value::*, *};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const MAX_STACK_SIZE: usize = 64;

pub type VmFrame = Vec<(RefType, VmContextEntryRef)>;
pub type VmContextEntry = VmFunctionTable;
pub type VmContextEntryRef = Rc<RefCell<VmFunctionTable>>;
pub type VmContextTable = Vec<VmFunctionTable>;
pub type VmFunctionParameters = Option<TupleType>;
pub type VmFunctionVirtual = Box<Expr>;
pub type VmFunctionNative = fn(&VmFunctionParameters, &mut VmContext) -> VmResult;

// this macro creates a new function from a static reference and links it with
// an argument number specified in `$params` onto the function table.
// TODO: add support for constants/type constraints
macro_rules! native_funcs {
    ($ctx:expr, $name:expr, $params:expr, $func:ident) => {
        let params = (0..$params)
            .map(|i| Expr::Ref(format!("x{}", i)))
            .collect::<Vec<_>>();
        let params = if params.is_empty() {
            None
        } else {
            Some(params)
        };
        if let Some(entry) = $ctx.get($name) {
            let table = &mut *(entry.borrow_mut());
            table.lookup_set(&params, VmFunction::Native($func));
        } else {
            let mut table = VmFunctionTable::new();
            table.lookup_set(&params, VmFunction::Native($func));
            $ctx.insert($name.to_string(), Rc::new(RefCell::new(table)));
        }
    };
}

#[derive(Clone, Debug)]
pub struct VmFunctionTable
{
    read: Option<VmFunction>,
    overloads: Option<Vec<VmFunctionOverload>>,
}

#[derive(Clone, Debug)]
pub struct VmFunctionOverload
{
    pub definition: TupleType,
    pub implementation: VmFunction,
}

impl VmFunctionOverload
{
    pub fn new(definition: TupleType, implementation: VmFunction) -> Self
    {
        Self {
            definition,
            implementation,
        }
    }
}

impl VmFunctionTable
{
    pub fn new() -> Self
    {
        Self {
            read: None,
            // TODO: this should evaluate lazily
            overloads: Some(vec![]),
        }
    }

    pub fn with_virtual(mut self, params: &VmFunctionParameters, expr: Expr) -> Self
    {
        self.lookup_set(params, VmFunction::Virtual(Box::new(expr)));
        self
    }

    pub fn with_native(mut self, params: &VmFunctionParameters, func: VmFunctionNative) -> Self
    {
        self.lookup_set(params, VmFunction::Native(func));
        self
    }

    pub fn with_bytecode(mut self, params: &VmFunctionParameters, co: CodeObject) -> Self
    {
        self.lookup_set(params, VmFunction::ByteCode(co));
        self
    }

    pub fn read(&self) -> Option<&VmFunction>
    {
        self.read.as_ref()
    }

    pub fn set_read(&mut self, read: VmFunction)
    {
        self.read = Some(read);
    }

    pub fn lookup(&self, params: &VmFunctionParameters)
        -> Option<(Option<TupleType>, &VmFunction)>
    {
        if let Some(params) = params {
            // TODO: optimize lookup
            lookup_func(self.overloads.as_ref().unwrap(), params)
        } else {
            self.read.as_ref().and_then(|r| Some((None, r)))
        }
    }

    pub fn lookup_set(&mut self, definition: &VmFunctionParameters, implementation: VmFunction)
    {
        if let Some(definition) = definition.as_ref() {
            // TODO: implement insert lookup
            if let Some(bucket) = lookup_func_mut(self.overloads.as_mut().unwrap(), definition) {
                *bucket = implementation;
            } else {
                let overload = VmFunctionOverload::new(definition.clone(), implementation);
                // TODO: this should insert in a sorted order for faster/more precise lookup
                self.overloads.as_mut().unwrap().push(overload);
            }
        } else {
            self.read = Some(implementation);
        }
    }
}

#[derive(Clone)]
pub enum VmFunction
{
    Virtual(VmFunctionVirtual),
    Native(VmFunctionNative),
    ByteCode(CodeObject),
}

// search expression to be updated
pub fn lookup_func_mut<'t>(
    table: &'t mut Vec<VmFunctionOverload>,
    params: &TupleType,
) -> Option<&'t mut VmFunction>
{
    let plen = params.len();

    info!("table: {:?}", table);
    info!("looking up mut: {:?}", params);

    for entry in table
        .iter_mut()
        .filter(|entry| entry.definition.len() == plen)
    {
        if entry.definition.iter().zip(params).all(|pair| match pair {
            (Value(e), Value(g)) => e == g,
            (Ref(_), Ref(_)) => true,
            _ => false,
        }) {
            return Some(&mut entry.implementation);
        }
    }

    None
}

// search expression for execution
pub fn lookup_func<'t>(
    table: &'t Vec<VmFunctionOverload>,
    params: &TupleType,
) -> Option<(Option<TupleType>, &'t VmFunction)>
{
    info!("table: {:?}", table);
    info!("looking up: {:?}", params);
    let plen = params.len();
    'table: for entry in table
        .iter()
        .filter(|overload| overload.definition.len() == plen)
    {
        for pair in entry.definition.iter().zip(params) {
            match pair {
                (Value(e), Value(g)) if e == g => {}
                (Ref(_), _) => {}
                _ => continue 'table,
            }
        }
        return Some((Some(entry.definition.clone()), &entry.implementation));
    }
    None
}

impl std::fmt::Debug for VmFunction
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Virtual(n) => write!(f, "{:?}", n),
            Native(_) => write!(f, "<native>"),
            ByteCode(co) => write!(f, "{:?}", co),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmContext
{
    // TODO: use DoS-unsafe HashMap implementation here (FNV maybe?)
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

        native_funcs!(ctx.map, "pi", 0, vm_func_pi);
        native_funcs!(ctx.map, "e", 0, vm_func_e);
        native_funcs!(ctx.map, "if", 3, vm_func_if);
        native_funcs!(ctx.map, "sqrt", 1, vm_func_sqrt);
        native_funcs!(ctx.map, "sqrt", 2, vm_func_sqrt);
        native_funcs!(ctx.map, "log", 1, vm_func_log);
        native_funcs!(ctx.map, "log", 2, vm_func_log);
        native_funcs!(ctx.map, "assert", 1, vm_func_assert);

        // the reimplementation of `do`, `print`, and `println` would have
        // required another interface for variadic functions. as parameters
        // for such a use case could easily be replaced by passing a `Set`,
        // no further effort will be spent on implementing them variadically.

        ctx
    }
}

impl VmContext
{
    pub fn push_frame(&mut self, frame: VmFrame)
    {
        info!("pushing frame: {:?}", frame);
        self.stack.push(frame);
        assert!(self.stack.len() < MAX_STACK_SIZE, "stack size exceeded");
    }

    pub fn pop_frame(&mut self) -> bool
    {
        let last = self.stack.pop();
        info!("pushing frame: {:?}", last);
        last.is_some()
    }

    pub fn get(&self, name: &RefType) -> Option<VmContextEntryRef>
    {
        if let Some(current_frame) = self.stack.last() {
            for (var, val) in current_frame.iter() {
                if var == name {
                    info!("stack lookup: {:?}", val);
                    return Some(val.clone());
                }
            }
        }
        self.map.get(name).map(|r| r.clone())
    }

    pub fn set(&mut self, name: &RefType, entry: VmContextEntry)
    {
        self.map.insert(name.clone(), Rc::new(RefCell::new(entry)));
    }

    // deprecated! consider storing byte code via `set_bytecode` instead
    pub fn set_virtual(&mut self, name: &RefType, params: &VmFunctionParameters, expr: Expr)
    {
        info!("setting expr: {:?}", expr);
        if let Some(func) = self.map.get(name) {
            let table = &mut *(func.borrow_mut());
            table.lookup_set(params, VmFunction::Virtual(Box::new(expr)));
        } else {
            let table = VmFunctionTable::new().with_virtual(params, expr);
            self.map.insert(name.clone(), Rc::new(RefCell::new(table)));
        }
    }

    pub fn set_bytecode(&mut self, name: &RefType, params: &VmFunctionParameters, co: CodeObject)
    {
        info!("setting bytecode: {:?}", co);
        if let Some(func) = self.map.get(name) {
            let table = &mut *(func.borrow_mut());
            table.lookup_set(params, VmFunction::ByteCode(co));
        } else {
            let table = VmFunctionTable::new().with_bytecode(params, co);
            self.map.insert(name.clone(), Rc::new(RefCell::new(table)));
        }
    }
}

fn vm_func_pi(params: &VmFunctionParameters, _ctx: &mut VmContext) -> VmResult
{
    assert!(params.is_none());
    Ok(Numeric(std::f64::consts::PI))
}

fn vm_func_e(params: &VmFunctionParameters, _ctx: &mut VmContext) -> VmResult
{
    assert!(params.is_none());
    Ok(Numeric(std::f64::consts::E))
}

fn vm_func_sqrt(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    let params = run_tuple_exprs(params.as_ref().unwrap(), ctx)?;
    let mut params = params.iter();
    match (params.next(), params.next()) {
        (Some(Value(Numeric(from))), None) => Ok(Numeric(from.sqrt())),
        (Some(Value(Numeric(from))), Some(Value(Numeric(n)))) => Ok(Numeric(from.powf(1. / n))),
        (_, _) => Err("function `sqrt` expected some paramaters".to_string()),
    }
}

fn vm_func_log(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    let params = run_tuple_exprs(params.as_ref().unwrap(), ctx)?;
    let mut params = params.iter();
    match (params.next(), params.next()) {
        (Some(Value(Numeric(from))), None) => Ok(Numeric(from.log10())),
        (Some(Value(Numeric(from))), Some(Value(Numeric(n)))) => Ok(Numeric(from.log(*n))),
        (_, _) => Err("function `log` expected some paramaters".to_string()),
    }
}

fn vm_func_if(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    let params = params.as_ref().unwrap();
    assert_eq!(params.len(), 3);
    run_with_ctx(&params.get(0).unwrap(), ctx).and_then(|cond| {
        if (&cond).into() {
            run_with_ctx(&params.get(1).unwrap(), ctx)
        } else {
            run_with_ctx(&params.get(2).unwrap(), ctx)
        }
    })
}

fn vm_func_assert(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    let params = params.as_ref().unwrap();
    assert_eq!(params.len(), 1);
    match run_with_ctx(&params.get(0).unwrap(), ctx) {
        Ok(Logical(true)) => {}
        // TODO: print place of occurrence
        _ => panic!("assert condition violated"),
    }
    Ok(Nil)
}

/*
fn vm_func_do(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    if let Some(params) = params {
        for param in params {
            run_with_ctx(&param, ctx)?;
        }
    }
    Ok(Nil)
}

fn vm_func_print(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    if let Some(params) = params {
        let params = run_tuple_exprs(params, ctx)?;
        for param in params {
            print!("{:?}", param);
        }
    }
    Ok(Nil)
}

fn vm_func_println(params: &VmFunctionParameters, ctx: &mut VmContext) -> VmResult
{
    vm_func_print(params, ctx)?;
    println!();
    Ok(Nil)
}

*/
