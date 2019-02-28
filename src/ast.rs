use crate::vm::context::*;

pub type NumType = f64;
pub type LogType = bool;
pub type RefType = String;
pub type TupleType = Vec<Expr>;
pub type SetType = Vec<Expr>;

#[derive(Clone, Debug)]
pub enum Value
{
    Nil,
    Numeric(NumType),
    Logical(LogType),
    Tuple(TupleType),
    Set(SetType),
}

impl std::convert::From<NumType> for Value
{
    fn from(n: NumType) -> Self
    {
        Value::Numeric(n)
    }
}

impl std::convert::From<&Value> for NumType
{
    fn from(v: &Value) -> Self
    {
        match v {
            Value::Numeric(n) => *n,
            _ => unimplemented!(),
        }
    }
}

impl std::convert::From<LogType> for Value
{
    fn from(l: LogType) -> Self
    {
        Value::Logical(l)
    }
}

impl std::convert::From<&Value> for LogType
{
    fn from(v: &Value) -> Self
    {
        match v {
            Value::Numeric(n) => *n != 0.,
            Value::Logical(l) => *l,
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
    Func(RefType, VmFunctionParameters),
}

impl std::fmt::Debug for Expr
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        match self {
            Expr::Value(v) => write!(f, "{:?}", v),
            Expr::Ref(r) => write!(f, "~{}", r),
            Expr::Comp(op, lhs, rhs) => write!(f, "Comp({:?}, {:?}, {:?})", op, lhs, rhs),
            Expr::Func(n, ls) => write!(f, "Func({:?}, {:?})", n, ls),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Rem,

    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,

    And,
    Or,

    Store,
}
