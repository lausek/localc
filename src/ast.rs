pub type NumType = f64;
pub type LogType = bool;
pub type RefType = String;
pub type SetType = Vec<Expr>;

#[derive(Debug, Clone)]
pub enum Value
{
    Numeric(NumType),
    Logical(LogType),
}

impl std::convert::From<NumType> for Value
{
    fn from(n: NumType) -> Self
    {
        Value::Numeric(n)
    }
}

impl std::convert::From<Value> for NumType
{
    fn from(n: Value) -> Self
    {
        match n {
            Value::Numeric(n) => n,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr
{
    Value(Value),
    Ref(RefType),
    //Set(SetType),
    Comp(Operator, Box<Expr>, Box<Expr>),
    // declaration or invocation
    Func(RefType, SetType),

    // TODO: is this needed?
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
