pub type NumType = f64;
pub type LogType = bool;
pub type RefType = String;
pub type TupleType = Vec<Expr>;

#[derive(Clone, Debug)]
pub enum Value
{
    Nil,
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

impl std::convert::From<Value> for LogType
{
    fn from(n: Value) -> Self
    {
        match n {
            Value::Numeric(n) => n != 0.,
            Value::Logical(l) => l,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub enum Expr
{
    Value(Value),
    Comp(Operator, Box<Expr>, Box<Expr>),

    Ref(RefType),
    // declaration or invocation
    Func(RefType, TupleType),
}

impl std::fmt::Debug for Expr
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        match self {
            Expr::Value(v) => write!(f, "{:?}", v),
            Expr::Ref(r) => write!(f, "~{:?}", r),
            Expr::Comp(op, lhs, rhs) => write!(f, "Comp({:?}, {:?}, {:?})", op, lhs, rhs),
            Expr::Func(n, ls) => write!(f, "Func({:?}, {:?})", n, ls),
        }
    }
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
