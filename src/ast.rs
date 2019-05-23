use super::*;

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
    Func(RefType, TupleType),
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

impl From<Expr> for lovm::gen::OpValue {
    fn from(v: Expr) -> Self {
        use lovm::gen::*;

        match v {
            Expr::Value(v) => Self::from(v),
            Expr::Ref(name) => Self::from(name.as_ref()),
            // TODO: add these
            Expr::Comp(op, lhs, rhs) => {
                let ty = match op {
                    Operator::Add => OperationType::Add,
                    Operator::Sub => OperationType::Sub,
                    Operator::Mul => OperationType::Mul,
                    Operator::Div => OperationType::Div,
                    Operator::Pow => OperationType::Pow,
                    Operator::Rem => OperationType::Rem,
                    Operator::Eq => OperationType::CmpEq,
                    Operator::Ne => OperationType::CmpNe,
                    Operator::Ge => OperationType::CmpGe,
                    Operator::Gt => OperationType::CmpGt,
                    Operator::Le => OperationType::CmpLe,
                    Operator::Lt => OperationType::CmpLt,
                    Operator::And => OperationType::And,
                    Operator::Or => OperationType::Or,
                    Operator::Store => OperationType::Ass,
                };
                let mut comp = Operation::new(ty);
                OpValue::Operation(comp.op(*lhs).op(*rhs).end())
            }
            Expr::Func(fname, args) => {
                let argc = args.len();
                let mut call = Operation::call(&fname);

                for arg in args.into_iter() {
                    call.op(arg);
                }

                call.op(argc);
                OpValue::Operation(call.end())
            }
        }
    }
}

impl From<Value> for lovm::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Numeric(n) => lovm::Value::F64(n),
            Value::Logical(t) => lovm::Value::T(t),
            _ => unimplemented!(),
        }
    }
}

impl From<NumType> for Value {
    fn from(n: NumType) -> Self {
        Value::Numeric(n)
    }
}

impl From<&Value> for NumType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Numeric(n) => *n,
            _ => unimplemented!(),
        }
    }
}

impl From<LogType> for Value {
    fn from(l: LogType) -> Self {
        Value::Logical(l)
    }
}

impl From<&Value> for LogType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Numeric(n) => *n != 0.,
            Value::Logical(l) => *l,
            _ => unimplemented!(),
        }
    }
}

impl From<&str> for Expr {
    fn from(from: &str) -> Self {
        Expr::Ref(from.to_string())
    }
}

impl<V> From<V> for Expr
where
    V: Into<Value>,
{
    fn from(v: V) -> Self {
        Expr::Value(v.into())
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
