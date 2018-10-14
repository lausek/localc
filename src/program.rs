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

pub fn execute_with_ctx(program: &Node, mut ctx: &Context)
    -> Result<Num, String>
{
    match program {
        Add(x, y) => Ok(execute(x)? + execute(y)?),
        Sub(x, y) => Ok(execute(x)? - execute(y)?),
        Mul(x, y) => Ok(execute(x)? * execute(y)?),
        Pow(x, y) => Ok(execute(x)?.powf(execute(y)?)),
        Sqrt(x)   => Ok(execute(x)?.sqrt()),
        Div(x, y) => {
            let arg2 = execute(y)?;
            if arg2 == 0 as Num {
                Err("division with 0".to_string())
            } else {
                Ok(execute(x)? / arg2)
            }
        },
        Value(n) => Ok(*n),
        Var(ref name) => {
            // TODO: implement variable lookup
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
    let mut ctx = HashMap::new();

    ctx.insert(String::from("pi"), Value(3.14159265));

    execute_with_ctx(program, &mut ctx)
}
