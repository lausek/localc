pub mod config;
pub mod context;

pub use self::{config::*, context::*};
use crate::{
    ast::*,
    compiler::{self, *},
    expr::ExprParser,
};

pub type VmValue = Value;
pub type VmError = String;
pub type VmResult = Result<VmValue, VmError>;

impl GenType
{
    pub fn new() -> Self
    {
        Self {
            constraints: SetType::new(),
            current: NumType::from(0.),
        }
    }

    pub fn contains(&mut self) {}

    pub fn next(&mut self, vm: &mut Vm) -> Option<Value>
    {
        loop {
            if self.constraints.iter().all(|expr| match vm.run(&expr) {
                Ok(Value::Logical(true)) => true,
                // TODO: raise errors if err(_)
                _ => false,
            }) {
                return Some(Value::Numeric(self.current));
            }
            self.current += 1.;
        }
    }
}

pub struct Vm
{
    config: VmConfig,
    ctx: VmContext,
    pub parser: ExprParser,
}

impl Vm
{
    pub fn new() -> Self
    {
        Self {
            config: VmConfig::new(),
            ctx: VmContext::new(),
            parser: ExprParser::new(),
        }
    }

    pub fn with_config(mut self, config: VmConfig) -> Self
    {
        self.config = config;
        self
    }

    pub fn with_stdlib() -> Self
    {
        Self {
            config: VmConfig::new(),
            ctx: VmContext::stdlib(),
            parser: ExprParser::new(),
        }
    }

    pub fn run_raw(&mut self, raw: &str) -> VmResult
    {
        let program = self.parser.parse(raw).unwrap();
        self.run(&program)
    }

    pub fn run_bytecode(&mut self, co: &CodeObject) -> VmResult
    {
        run_bytecode_with_ctx(co, &mut self.ctx)
    }

    pub fn run(&mut self, expr: &Expr) -> VmResult
    {
        match compiler::compile(expr) {
            Ok(co) => run_bytecode_with_ctx(&co, &mut self.ctx),
            Err(e) => Err(e),
        }
    }

    pub fn optimize(&self, expr: &mut Expr) -> Result<(), String>
    {
        optimize(expr)
    }

    pub fn config(&self) -> &VmConfig
    {
        &self.config
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
            if let Some(params) = params {
                for param in params {
                    optimize(param).unwrap();
                }
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
    // TODO: to lower recursion there must be a special bytecode version that
    // 		 translates the expression to a sequence of steps like:
    //		 	push arg1
    //		 	push arg2
    // 			op +
    // where `op` pops the last two values off the stack leaving the operations
    // result in place. This fixes an old misunderstanding where the vm was expected
    // to be run on an ast instead of a bytecode list and should have been
    // implemented in the parser as such.

    match expr {
        Expr::Value(v) => Ok(v.clone()),
        Expr::Ref(name) => run_lookup(name, ctx),
        Expr::Func(name, params) => run_function(name, params, ctx),
        Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
            box Expr::Func(name, params) => {
                ctx.set_virtual(name, &params, expr.clone());
                Ok(Value::Nil)
            }
            box Expr::Ref(name) => {
                ctx.set_virtual(name, &None, expr.clone());
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

pub fn run_bytecode_with_ctx(co: &CodeObject, ctx: &mut VmContext) -> VmResult
{
    let mut params_reg: VmFunctionParameters = None;
    let mut stack = vec![];

    info!("bytecode!");
    for instruction in co.iter() {
        match instruction {
            Instruction::Params(params) => params_reg = Some(params.clone()),
            Instruction::Call(name) => {
                info!("calling params with {:?}", params_reg);
                stack.push(run_function(name, &params_reg.take(), ctx)?);
            }
            Instruction::Move(name, expr) => {
                info!("setting `{:?}` with {:?}", name, params_reg);
                ctx.set_virtual(name, (params_reg.take(), expr.clone()));
                stack.push(Value::Nil);
            }
            Instruction::Load(name) => stack.push(run_lookup(name, ctx)?),
            Instruction::Push(v) => stack.push(v.clone()),
            Instruction::Pop => assert!(stack.pop().is_some()),
            _ => {
                let arg2 = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
                let op = Operator::from(instruction);
                stack.push(run_operation(&op, &arg1, &arg2)?);
                info!("exec {:?} with ({:?}, {:?})", op, arg1, arg2);
            }
        }
    }

    Ok(stack.pop().unwrap().clone())
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
    // TODO: ctx.get(name): Opt<VmFunctionTable> -> VmFunctionTable.read.unwrap
    if let Some(entry) = ctx.get(name) {
        match (&*(entry.borrow())).lookup(&None).unwrap().1 {
            //match &*(entry.borrow()) {
            VmFunction::Virtual(expr) => {
                info!("resulted in: {:?}", expr);
                run_with_ctx(&expr, ctx)
            }
            VmFunction::Native(func) => func(&None, ctx),
        }
    } else {
        Err(format!("function `{}` is unknown", name))
    }
}

pub fn run_function(name: &RefType, params: &VmFunctionParameters, ctx: &mut VmContext)
    -> VmResult
{
    info!("function: {} ({:?})", name, params);
    if let Some(entry) = ctx.get(name) {
        match (&*(entry.borrow())).lookup(params) {
            Some((args, func)) => match func {
                VmFunction::Virtual(expr) => {
                    let params = if let Some(p) = params {
                        let e = run_tuple_exprs(p, ctx)?;
                        Some(e)
                    } else {
                        None
                    };
                    info!("resulted in: {:?}", expr);
                    push_ctx_params(ctx, &args, &params);
                    let result = run_with_ctx(&expr, ctx);
                    pop_ctx_params(ctx);
                    result
                }
                VmFunction::Native(func) => func(&params, ctx),
            },
            _ => Err("unexpected arguments".to_string()),
        }
    /*
            match &*(entry.borrow()) {
                VmFunction::Virtual(table) => {
                    let params = if let Some(p) = params {
                        let e = run_tuple_exprs(p, ctx)?;
                        Some(e)
                    } else {
                        None
                    };
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
    */
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

fn push_ctx_params(ctx: &mut VmContext, args: &VmFunctionParameters, params: &VmFunctionParameters)
{
    use std::cell::RefCell;
    use std::rc::Rc;
    let frame = match (args, params) {
        (Some(args), Some(params)) => {
            assert_eq!(args.len(), params.len());
            args.iter()
                .zip(params)
                .filter_map(|(arg, param)| {
                    let expr = Box::new(param.clone());
                    match param {
                        Expr::Ref(name) => {
                            let mut table = VmFunctionTable::new();
                            table.lookup_add(&None);
                            Some((name.clone(), Rc::new(RefCell::new(table))))
                        }
                        Expr::Value(_) => None,
                        _ => unimplemented!(),
                    }
                })
                .collect::<VmFrame>()
        }
        (_, _) => vec![],
    };
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
            (Value::Logical(lhs), Value::Logical(rhs)) => lhs.partial_cmp(rhs),
            _ => unimplemented!(),
        }
    }
}
