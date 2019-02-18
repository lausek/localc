pub mod context;
pub mod node;
pub mod num;

pub use self::context::Context;
pub use self::node::Node;
pub use self::num::Num;

use self::context::is_node_assignable;
use self::node::{Node::*, Truth};

//use parser::parse;

#[derive(Clone, Debug)]
pub enum Computation
{
    Numeric(Num),
    Logical(Truth),
    Set(Vec<Computation>),
}

impl std::fmt::Display for Computation
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        use self::Computation::*;
        match self {
            Numeric(n) => write!(f, "{}", n)?,
            Logical(t) => write!(f, "{}", t)?,
            Set(vals) => {
                write!(f, "{{")?;
                if let Some(v) = vals.get(0) {
                    write!(f, "{}", v)?;
                }
                for v in vals.iter().skip(1) {
                    write!(f, ",{}", v)?;
                }
                write!(f, "}}")?
            }
        }
        Ok(())
    }
}

pub type ComputationResult<V> = Result<V, String>;

pub fn execute(program: &Node) -> ComputationResult<Computation>
{
    let mut ctx = Context::default();
    execute_with_ctx(program, &mut ctx)
}

pub fn execute_script(script: std::fs::File) -> ComputationResult<Computation>
{
    use std::io::{BufRead, BufReader};
    let mut ctx = Context::default();
    let mut iter = BufReader::new(script).lines().into_iter().peekable();

    while let Some(line) = iter.next() {
        if line.is_err() {
            return Err("line could not be read from script".to_string());
        }

        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }

        /*
        let program = parse(line)?;
        match execute_with_ctx(&program, &mut ctx) {
            res @ Ok(_) => {
                if let None = iter.peek() {
                    return res;
                }
            }
            e @ Err(_) => return e,
        }
        */
    }

    Err("script is empty".to_string())
}

pub fn execute_with_ctx(program: &Node, ctx: &mut Context) -> ComputationResult<Computation>
{
    use self::Computation::*;
    match program {
        Add(x, y) | Sub(x, y) | Mul(x, y) | Div(x, y) | Pow(x, y) | Mod(x, y) | Idx(x, y) => {
            let arg1 = execute_with_ctx(x, ctx)?;
            let arg2 = execute_with_ctx(y, ctx)?;

            match (arg1, arg2) {
                (Numeric(arg1), Numeric(arg2)) => compute_numeric(&program, arg1, arg2),
                (Set(arg1), index) => match index {
                    Numeric(arg2) => match arg1.get(arg2.as_usize()) {
                        Some(item) => Ok((*item).clone()),
                        _ => Err(format!("cannot resolve index `{}` for `{:?}`", arg2, arg1)),
                    },
                    _ => Err(format!("invalid index `{}` for `{:?}`", index, arg1)),
                },
                _ => unimplemented!(),
            }
        }
        Eq(x, y) | Ne(x, y) | Gt(x, y) | Lt(x, y) | Ge(x, y) | Le(x, y) | Or(x, y) | And(x, y) => {
            let arg1 = execute_with_ctx(x, ctx)?;
            let arg2 = execute_with_ctx(y, ctx)?;
            compute_logical(&program, arg1, arg2)
        }
        Mov(x, y) => {
            // FIXME: find alternative for `box`
            if let box Var(ref name) = x {
                ctx.set(name.clone(), y.clone())?;
                Ok(Logical(true))
            } else if let box Func(ref name, args) = x {
                if !is_node_assignable(&x) {
                    return Err(format!("cannot assign to `{}`", x));
                }
                ctx.setf(
                    name.clone(),
                    (args.clone(), context::ContextFunction::Virtual(y.clone())),
                )?;
                // FIXME this should become `true`
                Ok(Logical(true))
            } else {
                Err(format!("cannot assign to `{:?}`", x))
            }
        }
        Var(ref name) => {
            if ctx.get(name).is_none() {
                return Err(format!("variable `{}` not declared", name));
            }
            // FIXME: `clone` should be avoided here
            let var = ctx.get(name).unwrap().clone();
            Ok(execute_with_ctx(&var, ctx)?)
        }
        NVal(ref n) => Ok(Numeric(*n)),
        TVal(ref n) => Ok(Logical(*n)),
        SVal(ref vals) => {
            let mut result = Vec::new();
            for v in vals {
                // FIXME: don't call execute if value is numeric or logical
                let part = execute_with_ctx(&v, ctx)?;
                result.push(part);
            }
            Ok(Set(result))
        }
        Func(ref name, args) => {
            if ctx.getf(name).is_none() {
                return Err(format!("function `{}` not declared", name));
            }
            // FIXME: `clone` should be avoided here
            let (def, algo) = ctx.getf(name).unwrap().clone();

            if def.len() != args.len() {
                return Err(format!(
                    "invalid function call. expected `{}` got `{}` arguments.",
                    def.len(),
                    args.len()
                ));
            }

            let mut temp_ctx = ctx.clone();
            build_new_ctx(&mut temp_ctx, &def, &args)?;

            match algo {
                context::ContextFunction::Virtual(node) => {
                    Ok(execute_with_ctx(&node, &mut temp_ctx)?)
                }
                context::ContextFunction::Native(func) => func(ctx, args),
            }
        }
    }
}

fn compute_numeric(op: &Node, arg1: Num, arg2: Num) -> ComputationResult<Computation>
{
    use self::Computation::*;
    match op {
        Add(_, _) => Ok(Numeric(arg1 + arg2)),
        Sub(_, _) => Ok(Numeric(arg1 - arg2)),
        Mul(_, _) => Ok(Numeric(arg1 * arg2)),
        Pow(_, _) => Ok(Numeric(arg1.powf(arg2))),
        Mod(_, _) => Ok(Numeric(arg1 % arg2)),
        Div(_, _) => {
            if arg2 == Num::new(0.0) {
                Err("division with 0".to_string())
            } else {
                Ok(Numeric(arg1 / arg2))
            }
        }
        _ => unreachable!(),
    }
}

fn compute_logical(
    op: &Node,
    arg1: Computation,
    arg2: Computation,
) -> ComputationResult<Computation>
{
    use self::Computation::*;
    match (arg1, arg2) {
        (Numeric(arg1), Numeric(arg2)) => match op {
            Eq(_, _) => Ok(Logical(arg1 == arg2)),
            Ne(_, _) => Ok(Logical(arg1 != arg2)),
            Gt(_, _) => Ok(Logical(arg1 > arg2)),
            Lt(_, _) => Ok(Logical(arg1 < arg2)),
            Ge(_, _) => Ok(Logical(arg1 >= arg2)),
            Le(_, _) => Ok(Logical(arg1 <= arg2)),
            _ => panic!("compute_logical called without appropriate NodeBox"),
        },
        (Logical(arg1), Logical(arg2)) => match op {
            Eq(_, _) => Ok(Logical(arg1 == arg2)),
            Ne(_, _) => Ok(Logical(arg1 != arg2)),
            Or(_, _) => Ok(Logical(arg1 || arg2)),
            And(_, _) => Ok(Logical(arg1 && arg2)),
            _ => panic!("compute_logical called without appropriate NodeBox"),
        },
        _ => unreachable!(),
    }
}

fn build_new_ctx(ctx: &mut Context, def: &[Box<Node>], args: &[Box<Node>])
    -> ComputationResult<()>
{
    for (i, d) in def.iter().enumerate() {
        match d {
            box Var(name) => {
                let var = if let box Var(ref x) = args[i] {
                    ctx.get(x).unwrap().clone()
                } else {
                    args[i].clone()
                };
                ctx.set(name.clone(), var)?;
            }
            _ => return Err(format!("`{:?}` is not allowed in a function definition", d)),
        }
    }
    Ok(())
}
