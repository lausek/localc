pub mod context;
pub mod node;
pub mod num;

pub use self::num::Num;

use self::context::GenericContext;
use self::node::{Node, Node::*};

pub type ComputationResult<V> = Result<V, String>;

pub fn execute_with_ctx(program: &Node, ctx: &mut GenericContext) -> ComputationResult<Num>
{
    match program {
        Add(x, y) | Sub(x, y) | Mul(x, y) | Div(x, y) | Pow(x, y) => {
            let arg1 = execute_with_ctx(x, ctx)?;
            let arg2 = execute_with_ctx(y, ctx)?;

            match program {
                Add(_, _) => Ok(arg1 + arg2),
                Sub(_, _) => Ok(arg1 - arg2),
                Mul(_, _) => Ok(arg1 * arg2),
                Pow(_, _) => Ok(arg1.powf(arg2)),
                Div(_, _) => {
                    if arg2 == Num::new(0.0) {
                        Err("division with 0".to_string())
                    } else {
                        Ok(arg1 / arg2)
                    }
                }
                _ => unreachable!(),
            }
        }
        Mov(x, y) => {
            // FIXME: find alternative for `box`
            if let box Var(ref name) = x {
                ctx.set(name.clone(), y.clone());
                Ok(execute_with_ctx(y, ctx)?)
            } else if let box Func(ref name, args) = x {
                ctx.setf(
                    name.clone(),
                    (args.clone(), context::ContextFunction::Virtual(y.clone())),
                );
                Ok(execute_with_ctx(y, ctx)?)
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
        Val(ref n) => Ok(*n),
        Func(ref name, args) => {
            if ctx.getf(name).is_none() {
                return Err(format!("function `{}` not declared", name));
            }
            // FIXME: `clone` should be avoided here
            let (def, algo) = ctx.getf(name).unwrap().clone();
            let mut temp_ctx = ctx.clone();

            if def.len() != args.len() {
                return Err(format!(
                    "invalid function call. expected `{}` got `{}` arguments.",
                    def.len(),
                    args.len()
                ));
            }

            for (i, d) in def.iter().enumerate() {
                match d {
                    box Var(name) => temp_ctx.set(name.clone(), args[i].clone()),
                    _ => return Err(format!("`{:?}` is not allowed in a function definition", d)),
                }
            }

            match algo {
                context::ContextFunction::Virtual(node) => {
                    Ok(execute_with_ctx(&node, &mut temp_ctx)?)
                }
                context::ContextFunction::Native(func) => func(ctx, args),
            }
        }
    }
}

pub fn execute(program: &Node) -> ComputationResult<Num>
{
    let mut ctx = Default::default();
    execute_with_ctx(program, &mut ctx)
}
