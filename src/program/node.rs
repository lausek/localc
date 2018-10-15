use super::Num;
use self::Node::*;

pub type NodeBox = Box<Node>;

#[derive(Clone, Debug)]
pub enum Node {
    Add(NodeBox, NodeBox),
    Sub(NodeBox, NodeBox),
    Mul(NodeBox, NodeBox),
    Div(NodeBox, NodeBox),
    Pow(NodeBox, NodeBox),

    // assignment
    Equ(NodeBox, NodeBox),

    // FIXME: rename this to `Val`
    Value(Num),
    
    // identifier
    Var(String),
    // identifier, arguments
    FCall(String, Vec<NodeBox>),
    FDef(String, Vec<String>),

    // FIXME: should be replaced by context functions in near future
    Sqrt(NodeBox),
}

impl std::fmt::Display for Node 
{
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        match self {
            Add(x, y) => write!(f, "{} + {}", x, y),
            Sub(x, y) => write!(f, "{} - {}", x, y),
            Mul(x, y) => write!(f, "{} * {}", x, y),
            Div(x, y) => write!(f, "{} / {}", x, y),
            Pow(x, y) => write!(f, "{}^{}", x, y),
            Equ(x, y) => write!(f, "{} = {}", x, y),
            Var(x)    => write!(f, "{}", x),
            Value(x)  => write!(f, "{}", x),
            _ => Ok(()), 
        }
    }
}
