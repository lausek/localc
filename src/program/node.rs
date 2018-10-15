use super::Num;

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
