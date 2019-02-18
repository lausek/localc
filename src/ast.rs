// TODO: move program::Node here
pub type NumType = f64;
pub type LogType = bool;
pub type RefType = String;
pub type SetType = Vec<Expr>;

#[derive(Debug, Clone)]
pub enum Expr
{
    Numeric(NumType),
    Logical(LogType),
    Ref(RefType),
    Set(SetType),
    Comp(Operator, Box<Expr>, Box<Expr>),
    // declaration or invocation
    Func(RefType, SetType),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Operator
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}
