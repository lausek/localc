use self::Node::*;
use super::Num;

pub type NodeBox = Box<Node>;

#[derive(Clone, Debug)]
pub enum Node
{
    Add(NodeBox, NodeBox),
    Sub(NodeBox, NodeBox),
    Mul(NodeBox, NodeBox),
    Div(NodeBox, NodeBox),
    Pow(NodeBox, NodeBox),

    // assignment `:=`
    Mov(NodeBox, NodeBox),

    Val(Num),

    // identifier
    Var(String),
    // identifier, arguments
    Func(String, Vec<NodeBox>),
}

impl std::fmt::Display for Node
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Add(x, y) => write!(f, "{} + {}", x, y),
            Sub(x, y) => write!(f, "{} - {}", x, y),
            Mul(x, y) => write!(f, "{} * {}", x, y),
            Div(x, y) => write!(f, "{} / {}", x, y),
            Pow(x, y) => write!(f, "{}^({})", x, y),
            Mov(x, y) => write!(f, "{} := {}", x, y),
            Val(x) => write!(f, "{}", x),
            Var(x) => write!(f, "{}", x),
            Func(x, y) => {
                let args = y.iter().enumerate().fold(String::new(), |mut acc, (i, x)| {
                    if 0 < i {
                        acc.push(',');
                    }
                    acc.push_str(&format!("{}", x));
                    acc
                });
                write!(f, "{}({})", x, args)
            }
        }
    }
}
