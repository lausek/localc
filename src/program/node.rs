use self::Node::*;
use super::Num;

pub type Truth = bool;
pub type Identifier = String;
pub type NodeBox = Box<Node>;

#[derive(Clone, Debug)]
pub enum Node
{
    // numerical
    Add(NodeBox, NodeBox),
    Sub(NodeBox, NodeBox),
    Mul(NodeBox, NodeBox),
    Div(NodeBox, NodeBox),
    Pow(NodeBox, NodeBox),

    // logical
    Eq(NodeBox, NodeBox),
    Ne(NodeBox, NodeBox),
    Gt(NodeBox, NodeBox),
    Lt(NodeBox, NodeBox),
    Ge(NodeBox, NodeBox),
    Le(NodeBox, NodeBox),
    Or(NodeBox, NodeBox),
    And(NodeBox, NodeBox),

    // assignment `=`
    Mov(NodeBox, NodeBox),

    // indexing
    Idx(NodeBox, NodeBox),

    NVal(Num),
    TVal(Truth),
    SVal(Vec<NodeBox>),

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
            // numerical
            Add(x, y) => write!(f, "{} + {}", x, y)?,
            Sub(x, y) => write!(f, "{} - {}", x, y)?,
            Mul(x, y) => write!(f, "{} * {}", x, y)?,
            Div(x, y) => write!(f, "{} / {}", x, y)?,
            Pow(x, y) => write!(f, "{}^({})", x, y)?,
            Mov(x, y) => write!(f, "{} = {}", x, y)?,
            // logical
            Eq(x, y) => write!(f, "{} == {}", x, y)?,
            Ne(x, y) => write!(f, "{} != {}", x, y)?,
            Gt(x, y) => write!(f, "{} > {}", x, y)?,
            Lt(x, y) => write!(f, "{} < {}", x, y)?,
            Ge(x, y) => write!(f, "{} >= {}", x, y)?,
            Le(x, y) => write!(f, "{} <= {}", x, y)?,
            Or(x, y) => write!(f, "{} || {}", x, y)?,
            And(x, y) => write!(f, "{} && {}", x, y)?,

            Idx(x, y) => write!(f, "{}_{}", x, y)?,

            Var(x) => write!(f, "{}", x)?,
            Func(x, y) => {
                let args = y.iter().enumerate().fold(String::new(), |mut acc, (i, x)| {
                    if 0 < i {
                        acc.push(',');
                    }
                    acc.push_str(&format!("{}", x));
                    acc
                });
                write!(f, "{}({})", x, args)?
            }

            NVal(x) => write!(f, "{}", x)?,
            TVal(x) => write!(f, "{}", x)?,
            SVal(vals) => {
                if let Some(v) = vals.get(0) {
                    write!(f, "{}", v)?;
                }
                for v in vals.iter().skip(1) {
                    write!(f, ",{}", v)?;
                }
            }
        }
        Ok(())
    }
}

impl Node
{
    pub fn idents(&self) -> Option<Vec<Identifier>>
    {
        match self {
            Var(x) => Some(vec![x.clone()]),
            Func(x, args) => {
                let mut ils = vec![];
                ils.push(x.clone());
                for arg in args {
                    if let Some(deps) = arg.idents() {
                        ils.extend(deps);
                    }
                }
                Some(ils)
            }
            Add(lhs, rhs) | Sub(lhs, rhs) | Mul(lhs, rhs) | Div(lhs, rhs) | Pow(lhs, rhs) => {
                let mut ils = vec![];
                if let Some(lhs_deps) = lhs.idents() {
                    ils.extend(lhs_deps);
                }
                if let Some(rhs_deps) = rhs.idents() {
                    ils.extend(rhs_deps);
                }
                Some(ils)
            }
            Mov(_, rhs) => rhs.idents(),
            _ => None,
        }
    }
}
