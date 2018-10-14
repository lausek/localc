use std::collections::HashMap;

use self::Node::*;

pub type Num        = f64;
pub type NodeBox    = Box<Node>;
pub type Context    = HashMap<String, Node>;

#[derive(Clone, Debug)]
pub enum Node {
    Add(NodeBox, NodeBox),
    Sub(NodeBox, NodeBox),
    Mul(NodeBox, NodeBox),
    Div(NodeBox, NodeBox),

    // FIXME: rename this to `Val`
    Value(Num),

    Var(String),
    Func(NodeBox),

    // FIXME: should be replaced by context functions in near future
    Pow(NodeBox, NodeBox),
    Sqrt(NodeBox),
}

fn get_standard_ctx()
    -> Context
{
    let mut ctx = HashMap::new();
    ctx.insert(String::from("pi"), Value(3.14159265));
    ctx
}

pub fn execute_with_ctx(program: &Node, mut ctx: &Context)
    -> Result<Num, String>
{
    match program {
        Add(x, y) | Sub(x, y) |
        Mul(x, y) | Div(x, y) |
        Pow(x, y) => {
            let arg1 = execute(x)?;
            let arg2 = execute(y)?;

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
        Sqrt(x)  => Ok(execute(x)?.sqrt()),
        Value(n) => Ok(*n),
        Var(ref name) => {
            if let Some(var) = ctx.get(name) {
                Ok(execute(var)?)
            } else {
                return Err(format!("variable `{}` not declared", name));
            }
        },
        Func(_) => {
            unimplemented!();
        },
        _ => unreachable!(),
    }
}

pub fn execute(program: &Node)
    -> Result<Num, String>
{
    let mut ctx = get_standard_ctx();
    execute_with_ctx(program, &mut ctx)
}
