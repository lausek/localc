use super::*;

use lovm::*;

use std::cell::RefCell;
use std::rc::Rc;

// >>>>>> begin of migrated types
pub type VmFrame = Vec<(RefType, VmContextEntryRef)>;
pub type VmContextEntry = VmFunctionTable;
pub type VmContextEntryRef = Rc<RefCell<VmFunctionTable>>;
pub type VmContextTable = Vec<VmFunctionTable>;
pub type VmFunction = gen::FunctionBuilder;
pub type VmFunctionParameters = TupleType;
pub type VmFunctionOverload = TupleType;

#[derive(Clone, Debug)]
pub struct VmFunctionTable {
    read: Option<VmFunction>,
    overloads: Option<Vec<VmFunctionOverload>>,
}
// <<<<<< end of migrated types

pub type NumType = f64;
pub type LogType = bool;
pub type RefType = String;
pub type TupleType = Vec<Expr>;
pub type SetType = TupleType;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Numeric(NumType),
    Logical(LogType),
    Tuple(TupleType),
    Set(SetType),
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match (self, other) {
            (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            (Value::Numeric(s), Value::Numeric(o)) => s.partial_cmp(o),
            (Value::Logical(s), Value::Logical(o)) => s.partial_cmp(o),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    Comp(Operator, Box<Expr>, Box<Expr>),

    Ref(RefType),
    // declaration or invocation
    Func(RefType, VmFunctionParameters),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
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

impl std::convert::From<NumType> for Value {
    fn from(n: NumType) -> Self {
        Value::Numeric(n)
    }
}

impl std::convert::From<&Value> for NumType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Numeric(n) => *n,
            _ => unimplemented!(),
        }
    }
}

impl std::convert::From<LogType> for Value {
    fn from(l: LogType) -> Self {
        Value::Logical(l)
    }
}

impl std::convert::From<&Value> for LogType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Numeric(n) => *n != 0.,
            Value::Logical(l) => *l,
            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Numeric(n) => write!(f, "{}", n).unwrap(),
            Value::Logical(l) => write!(f, "{}", l).unwrap(),
            _ => unreachable!(),
        }
        Ok(())
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Expr::Value(v) => write!(f, "{:?}", v),
            Expr::Ref(r) => write!(f, "#{}", r),
            Expr::Comp(op, lhs, rhs) => write!(f, "Comp({:?}, {:?}, {:?})", op, lhs, rhs),
            Expr::Func(n, ls) => write!(f, "Func({:?}, {:?})", n, ls),
        }
    }
}
