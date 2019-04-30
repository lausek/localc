pub mod config;
pub mod context;

pub use self::{config::*, context::*};
use crate::{
    ast::*,
    compiler::{self, *},
    expr::ExprParser,
};

pub type VmError = String;
pub type VmResult = Result<Value, VmError>;

pub struct Vm {
    config: VmConfig,
    ctx: VmContext,
    pub parser: ExprParser,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            config: VmConfig::new(),
            ctx: VmContext::new(),
            parser: ExprParser::new(),
        }
    }

    pub fn with_config(mut self, config: VmConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_stdlib() -> Self {
        Self {
            config: VmConfig::new(),
            ctx: VmContext::stdlib(),
            parser: ExprParser::new(),
        }
    }

    pub fn config(&self) -> &VmConfig {
        &self.config
    }
}

pub fn run_with_ctx(expr: &Expr, ctx: &mut VmContext) -> VmResult {
    // to lower recursion there must be a special bytecode version that
    // translates the expression to a sequence of steps like:
    //		 	push arg1
    //		 	push arg2
    // 			op +
    // where `op` pops the last two values off the stack leaving the operations
    // result in place. this fixes an old misunderstanding where the vm was expected
    // to be run on an ast instead of a bytecode list and should have been
    // implemented in the parser therefore.

    match expr {
        Expr::Value(v) => match v {
            Value::Tuple(ls) => Ok(Value::Tuple(run_tuple_exprs(&ls, ctx)?)),
            Value::Set(ls) => Ok(Value::Set(run_tuple_exprs(&ls, ctx)?)),
            _ => Ok(v.clone()),
        },
        Expr::Ref(name) => run_lookup(name, ctx),
        Expr::Func(name, params) => run_function(name, params, ctx),
        Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
            box Expr::Func(name, params) => {
                ctx.set_virtual(name, &params, *rhs.clone());
                Ok(Value::Nil)
            }
            box Expr::Ref(name) => {
                ctx.set_virtual(name, &None, *rhs.clone());
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

pub fn run_bytecode_with_ctx(co: &CodeObject, ctx: &mut VmContext) -> VmResult {
    let mut params_reg: VmFunctionParameters = None;
    let mut stack = vec![];

    trace!("bytecode!");
    for instruction in co.iter() {
        match instruction {
            Instruction::Params(params) => params_reg = Some(params.clone()),
            Instruction::Call(name) => {
                trace!("calling params with {:?}", params_reg);
                stack.push(run_function(name, &params_reg.take(), ctx)?);
            }
            Instruction::Move(name, expr) => {
                trace!("setting `{:?}` with {:?}", name, params_reg);
                ctx.set_virtual(name, &params_reg.take(), *expr.clone());
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
                trace!("exec {:?} with ({:?}, {:?})", op, arg1, arg2);
            }
        }
    }

    Ok(stack.pop().unwrap().clone())
}

// executes the stored `read` operation for given reference `name`
pub fn run_lookup(name: &RefType, ctx: &mut VmContext) -> VmResult {
    trace!("lookup: {}", name);
    if let Some(entry) = ctx.get(name) {
        match (&*(entry.borrow())).lookup(&None).unwrap().1 {
            VmFunction::Virtual(expr) => {
                trace!("resulted in: {:?}", expr);
                run_with_ctx(&expr, ctx)
            }
            VmFunction::Native(func) => func(&None, ctx),
            VmFunction::ByteCode(co) => run_bytecode_with_ctx(&co, ctx),
        }
    } else {
        Err(format!("variable `{}` is unknown", name))
    }
}

// executes an applicable overload `name` with values in `params`. the overload
// matching is done inside the table itself.
pub fn run_function(
    name: &RefType,
    params: &VmFunctionParameters,
    ctx: &mut VmContext,
) -> VmResult {
    if let Some(entry) = ctx.get(name) {
        //let params = &Some(run_tuple_exprs(&params, ctx).unwrap());
        let params = if let Some(params) = params {
            Some(run_tuple_exprs(&params, ctx)?)
        } else {
            None
        };
        info!("function: {}({:?})", name, params);
        match (&*(entry.borrow())).lookup(&params) {
            Some((args, func)) => match func {
                VmFunction::Virtual(expr) => {
                    /*
                    let params = if let Some(p) = params {
                        let e = run_tuple_exprs(p, ctx)?;
                        Some(e)
                    } else {
                        None
                    };
                    */
                    trace!("resulted in: {:?}", expr);
                    push_ctx_params(ctx, &args, &params);
                    let result = run_with_ctx(&expr, ctx);
                    pop_ctx_params(ctx);
                    result
                }
                VmFunction::Native(func) => func(&params, ctx),
                VmFunction::ByteCode(co) => run_bytecode_with_ctx(&co, ctx),
            },
            _ => Err("unexpected arguments".to_string()),
        }
    } else {
        Err(format!("function `{}` is unknown", name))
    }
}

// simple wrapper for executing expressions stored in `Tuples`.
// TODO: rewrite this to a more generic approach as `Set` must evaluate this way
// aswell.
fn run_tuple_exprs(params: &TupleType, ctx: &mut VmContext) -> Result<TupleType, VmError> {
    let mut list = vec![];
    for param in params {
        let result = run_with_ctx(&param, ctx)?;
        list.push(Expr::Value(result));
    }
    Ok(list)
}

// transfers the parameters into a new frame on the call stack.
fn push_ctx_params(
    ctx: &mut VmContext,
    args: &VmFunctionParameters,
    params: &VmFunctionParameters,
) {
    use std::cell::RefCell;
    use std::rc::Rc;
    trace!("zipping: args({:?}), params({:?})", args, params);
    let frame = match (args, params) {
        (Some(args), Some(params)) => {
            // this must be guaranteed by the lookup in first place
            assert_eq!(args.len(), params.len());
            args.iter()
                .zip(params)
                .filter_map(|(arg, param)| match arg {
                    Expr::Ref(name) => match run_with_ctx(&param, ctx) {
                        Ok(value) => {
                            let table =
                                VmFunctionTable::new().with_virtual(&None, Expr::Value(value));
                            Some((name.clone(), Rc::new(RefCell::new(table))))
                        }
                        _ => panic!("not a value in function call"),
                    },
                    // we don't want to push values into the frame as they where already
                    // known at the time of declaration and redundant as such.
                    Expr::Value(_) => None,
                    _ => unreachable!(),
                })
                .collect::<VmFrame>()
        }
        (_, _) => vec![],
    };
    ctx.push_frame(frame);
}

// removes a frame from the stack, asserting that there was one. this function
// must be called immediately after executing a local call.
fn pop_ctx_params(ctx: &mut VmContext) {
    assert!(ctx.pop_frame());
}

// TODO: implement type coercion when `Str` arrives
impl std::cmp::PartialEq for Value {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Value::Numeric(lhs), Value::Numeric(rhs)) => lhs == rhs,
            (Value::Logical(lhs), Value::Logical(rhs)) => lhs == rhs,
            (Value::Tuple(lhs), Value::Tuple(rhs)) | (Value::Set(lhs), Value::Set(rhs)) => {
                if lhs.len() != rhs.len() {
                    return false;
                }
                lhs.iter().zip(rhs).all(|(l, r)| match (l, r) {
                    (Expr::Value(lhs), Expr::Value(rhs)) => lhs == rhs,
                    // TODO: if the tuple/set does not only contain constant values, evaluation
                    // 		 must happend before comparison somehow. requires ref to the context
                    _ => unimplemented!(),
                })
            }
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

// TODO: implement `Str`
impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        match (self, rhs) {
            (Value::Numeric(lhs), Value::Numeric(rhs)) => lhs.partial_cmp(rhs),
            (Value::Logical(lhs), Value::Logical(rhs)) => lhs.partial_cmp(rhs),
            _ => unimplemented!(),
        }
    }
}
