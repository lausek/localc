use self::Node::*;

pub type Num        = f64;
pub type NodeBox    = Box<Node>;
pub type Res        = Result<Num, &'static str>;

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

pub fn execute(program: &Node)
    -> Res
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
                Err("division with 0")
            } else {
                Ok(execute(x)? / arg2)
            }
        },
        Value(n)  => Ok(*n),
        Var(_) | Func(_) => {
            // TODO: implement variable lookup
            unimplemented!();
        },
        _ => unreachable!(),
    }
}
