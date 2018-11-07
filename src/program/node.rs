use self::Node::*;
use super::Num;

pub type Identifier = String;
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
    Var(Identifier),
    // identifier, arguments
    Func(Identifier, Vec<NodeBox>),
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

impl Node {

    pub fn idents(&self) -> Vec<Identifier>
    {
        let mut ils: Vec<Identifier> = vec![];
        match self {
            Var(x) => ils.push(x.clone()),
            Func(x, args) => {
                ils.push(x.clone());
                for arg in args {
                    ils.extend(arg.idents());
                }
            }, 
            Add(lhs, rhs) | Sub(lhs, rhs) |
            Mul(lhs, rhs) | Div(lhs, rhs) |
            Pow(lhs, rhs) => {
                ils.extend(lhs.idents());
                ils.extend(rhs.idents());
            },
            Mov(_, rhs) => {
                ils.extend(rhs.idents());
            },
            _ => {},
        }
        ils
    }

}
