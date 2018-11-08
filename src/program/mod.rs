pub mod context;
pub mod node;
pub mod num;

pub use self::num::Num;

use self::context::{is_node_assignable, Context};
use self::node::{Node, Node::*, Truth};

#[derive(Debug)]
pub enum Computation {
    Numeric(Num),
    Logical(Truth),
}

pub type ComputationResult<V> = Result<V, String>;

pub fn execute(program: &Node) -> ComputationResult<Computation>
{
    let mut ctx = Default::default();
    execute_with_ctx(program, &mut ctx)
}

pub fn execute_with_ctx(program: &Node, ctx: &mut Context) -> ComputationResult<Computation>
{
    use self::Computation::*;
    match program {
        Add(x, y) | Sub(x, y) | Mul(x, y) | Div(x, y) | Pow(x, y) => {
            let arg1 = execute_with_ctx(x, ctx)?;
            let arg2 = execute_with_ctx(y, ctx)?;

            match (arg1, arg2) {
                (Numeric(arg1), Numeric(arg2)) => compute_numeric(&program, arg1, arg2),
                _ => unimplemented!(),
            }
        }
        Mov(x, y) => {
            // FIXME: find alternative for `box`
            if let box Var(ref name) = x {
                ctx.set(name.clone(), y.clone())?;
                Ok(execute_with_ctx(y, ctx)?)
            } else if let box Func(ref name, args) = x {
                if !is_node_assignable(&x) {
                    return Err(format!("cannot assign to `{}`", x));
                }
                ctx.setf(
                    name.clone(),
                    (args.clone(), context::ContextFunction::Virtual(y.clone())),
                )?;
                // FIXME this should become `true`
                Ok(Numeric(Num::new(0.0)))
            } else {
                Err(format!("cannot assign to `{:?}`", x))
            }
        }
        Var(ref name) => {
            if ctx.get(name).is_none() {
                return Err(format!("variable `{}` not declared", name));
            }
            // FIXME: `clone` should be avoided here
            let var = ctx.get(name).unwrap().clone();
            Ok(execute_with_ctx(&var, ctx)?)
        }
        NVal(ref n) => Ok(Numeric(*n)),
        TVal(ref n) => Ok(Logical(*n)),
        Func(ref name, args) => {
            if ctx.getf(name).is_none() {
                return Err(format!("function `{}` not declared", name));
            }
            // FIXME: `clone` should be avoided here
            let (def, algo) = ctx.getf(name).unwrap().clone();

            if def.len() != args.len() {
                return Err(format!(
                    "invalid function call. expected `{}` got `{}` arguments.",
                    def.len(),
                    args.len()
                ));
            }

            let mut temp_ctx = ctx.clone();
            build_new_ctx(&mut temp_ctx, &def, &args)?;

            match algo {
                context::ContextFunction::Virtual(node) => {
                    Ok(execute_with_ctx(&node, &mut temp_ctx)?)
                }
                context::ContextFunction::Native(func) => func(ctx, args),
            }
        }
    }
}

fn compute_numeric(op: &Node, arg1: Num, arg2: Num)
    -> ComputationResult<Computation>
{
    use self::Computation::*;
    match op {
        Add(_, _) => Ok(Numeric(arg1 + arg2)),
        Sub(_, _) => Ok(Numeric(arg1 - arg2)),
        Mul(_, _) => Ok(Numeric(arg1 * arg2)),
        Pow(_, _) => Ok(Numeric(arg1.powf(arg2))),
        Div(_, _) => {
            if arg2 == Num::new(0.0) {
                Err("division with 0".to_string())
            } else {
                Ok(Numeric(arg1 / arg2))
            }
        }
        _ => unreachable!(),
    }
}

fn build_new_ctx(ctx: &mut Context, def: &[Box<Node>], args: &[Box<Node>])
    -> ComputationResult<()>
{
    for (i, d) in def.iter().enumerate() {
        match d {
            box Var(name) => {
                let var = if let box Var(ref x) = args[i] {
                    ctx.get(x).unwrap().clone()
                } else {
                    args[i].clone()
                };
                ctx.set(name.clone(), var)?;
            }
            _ => return Err(format!("`{:?}` is not allowed in a function definition", d)),
        }
    }
    Ok(())
}
