pub mod node;
pub mod context;

use self::node::{Node, Node::*};
use self::context::GenericContext;

pub type Num = f64;

pub fn execute_with_ctx(program: &Node, ctx: &mut GenericContext)
    -> Result<Num, String>
{
    match program {
        Add(x, y) | Sub(x, y) |
        Mul(x, y) | Div(x, y) |
        Pow(x, y) => {
            let arg1 = execute_with_ctx(x, ctx)?;
            let arg2 = execute_with_ctx(y, ctx)?;

            match program {
                Add(_, _) => Ok(arg1 + arg2),
                Sub(_, _) => Ok(arg1 - arg2),
                Mul(_, _) => Ok(arg1 * arg2),
                Pow(_, _) => Ok(arg1.powf(arg2)),
                Div(_, _) => {
                    if arg2 == 0 as Num {
                        Err("division with 0".to_string())
                    } else {
                        Ok(arg1 / arg2)
                    }
                },
                _ => unreachable!(),
            }
        },
        Equ(x, y) => {
            // FIXME: find alternative for `box`
            if let box Var(ref name) = x {
                ctx.set(name.clone(), y.clone());
                Ok(execute_with_ctx(y, ctx)?)
            } else {
                Err(format!("cannot assign to `{:?}`", x))
            }
        },
        Var(ref name) => {
            if ctx.get(name).is_none() {
                return Err(format!("variable `{}` not declared", name));
            }
            // FIXME: `clone` should be avoided here
            let var = ctx.get(name).unwrap().clone();
            Ok(execute_with_ctx(&var, ctx)?)
        },
        Value(n) => Ok(*n),
        FCall(ref name, args) => {
            unimplemented!();
        },
        Sqrt(x)  => Ok(execute_with_ctx(x, ctx)?.sqrt()),
        _ => unreachable!(),
    }
}

pub fn execute(program: &Node)
    -> Result<Num, String>
{
    let mut ctx = Default::default();
    execute_with_ctx(program, &mut ctx)
}
