use self::Node::*;

pub type Num = f64;
pub type Res = Result<Num, &'static str>;

#[derive(Clone, Debug)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Pow(Box<Node>, Box<Node>),
    Sqrt(Box<Node>),
    Value(Num),
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
        Value(n) => Ok(*n),
        _ => unreachable!(),
    }
}
