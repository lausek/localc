use crate::ast::*;

pub type CodeObject = Vec<Instruction>;
pub type CompileResult = Result<CodeObject, String>;

#[derive(Clone, Debug)]
pub enum Instruction
{
    // set the `params` local variable for calls
    Params(TupleType),
    // consumes the `params` variable leaving None in its place
    Call(RefType),
    // assigns the `Expr` specified in .1 to the name in .0
    // consumes the `params` variable leaving None in its place
    // in this case, the overloading for `params` will be used for futher
    // operation
    Move(RefType, Box<Expr>),
    // loads the current value of reference .0 onto the stack
    Load(RefType),
    // stores a constant on the stack
    Push(Value),
    // removes the last value on the stack
    Pop,

    // the following operations consume two arguments from the stack
    // leaving the operations result in place
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
}

pub fn compile(ast: &Expr) -> CompileResult
{
    let mut code_object = CodeObject::new();
    match ast {
        Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
            box Expr::Ref(name) => {
                code_object.extend(vec![Instruction::Move(name.clone(), rhs.clone())])
            }
            box Expr::Func(name, Some(params)) => code_object.extend(vec![
                Instruction::Params(params.clone()),
                Instruction::Move(name.clone(), rhs.clone()),
            ]),
            box Expr::Func(name, None) => {
                code_object.extend(vec![Instruction::Move(name.clone(), rhs.clone())])
            }
            _ => return Err("assignment not allowed".to_string()),
        },
        Expr::Comp(op, lhs, rhs) => {
            code_object.extend(compile(&lhs)?);
            code_object.extend(compile(&rhs)?);
            code_object.push(Instruction::from(op));
        }
        Expr::Func(name, Some(params)) => code_object.extend(vec![
            Instruction::Params(params.clone()),
            Instruction::Call(name.clone()),
        ]),
        Expr::Func(name, None) => code_object.extend(vec![Instruction::Call(name.clone())]),
        Expr::Ref(r) => code_object.push(Instruction::Load(r.clone())),
        Expr::Value(v) => code_object.push(Instruction::Push(v.clone())),
    }
    Ok(code_object)
}

impl From<&Operator> for Instruction
{
    fn from(from: &Operator) -> Self
    {
        match from {
            Operator::Add => Instruction::Add,
            Operator::Sub => Instruction::Sub,
            Operator::Mul => Instruction::Mul,
            Operator::Div => Instruction::Div,
            Operator::Pow => Instruction::Pow,
            Operator::Rem => Instruction::Rem,

            Operator::Eq => Instruction::Eq,
            Operator::Ne => Instruction::Ne,
            Operator::Ge => Instruction::Ge,
            Operator::Gt => Instruction::Gt,
            Operator::Le => Instruction::Le,
            Operator::Lt => Instruction::Lt,

            Operator::And => Instruction::And,
            Operator::Or => Instruction::Or,
            _ => unimplemented!(),
        }
    }
}

impl From<&Instruction> for Operator
{
    fn from(from: &Instruction) -> Self
    {
        match from {
            Instruction::Add => Operator::Add,
            Instruction::Sub => Operator::Sub,
            Instruction::Mul => Operator::Mul,
            Instruction::Div => Operator::Div,
            Instruction::Pow => Operator::Pow,
            Instruction::Rem => Operator::Rem,

            Instruction::Eq => Operator::Eq,
            Instruction::Ne => Operator::Ne,
            Instruction::Ge => Operator::Ge,
            Instruction::Gt => Operator::Gt,
            Instruction::Le => Operator::Le,
            Instruction::Lt => Operator::Lt,

            Instruction::And => Operator::And,
            Instruction::Or => Operator::Or,
            _ => unimplemented!(),
        }
    }
}
