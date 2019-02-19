pub type NumType = f64;
pub type LogType = bool;
pub type RefType = String;
pub type TupleType = Vec<Expr>;

#[derive(Clone, Debug)]
pub enum Value
{
    Empty,
    Numeric(NumType),
    Logical(LogType),
    Tuple(TupleType),
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

#[derive(Clone, Debug)]
pub enum Expr
{
    Value(Value),
    Comp(Operator, Box<Expr>, Box<Expr>),

    Ref(RefType),
    // declaration or invocation
    Func(RefType, TupleType),
}

#[derive(Clone, Debug)]
pub enum Operator
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,

    Equ,
}
