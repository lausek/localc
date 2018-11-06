use super::node::{Node::*, NodeBox};
use program::{ComputationResult, Num};
use std::collections::HashMap;

pub type Identifier = String;
pub type GenericContext = Context<Identifier, NodeBox>;
pub type Closure<K, V> = fn(&mut Context<K, V>, &Vec<V>) -> ComputationResult<Num>;

#[derive(Clone)]
pub enum ContextFunction<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::fmt::Display,
{
    Virtual(V),
    Native(Closure<K, V>),
}

impl<K, V> std::fmt::Debug for ContextFunction<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        use self::ContextFunction::*;
        match self {
            Virtual(n) => write!(f, "{}", n),
            _ => write!(f, "<native>"),
        }
        .unwrap();
        Ok(())
    }
}

impl<K, V> std::fmt::Display for ContextFunction<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        use self::ContextFunction::*;
        match self {
            Virtual(n) => write!(f, "{}", n),
            _ => write!(f, "<native>"),
        }
        .unwrap();
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Context<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::fmt::Display,
{
    vars: HashMap<K, V>,
    funcs: HashMap<K, (Vec<V>, ContextFunction<K, V>)>,
}

impl<K, V> Context<K, V>
where
    K: Eq + std::hash::Hash,
    V: std::fmt::Display,
{
    pub fn get(&self, key: &K) -> Option<&V>
    {
        self.vars.get(key)
    }

    pub fn getf(&self, key: &K) -> Option<&(Vec<V>, ContextFunction<K, V>)>
    {
        self.funcs.get(key)
    }

    pub fn set(&mut self, key: K, value: V)
    {
        self.vars.insert(key, value);
    }

    pub fn setf(&mut self, key: K, value: (Vec<V>, ContextFunction<K, V>))
    {
        self.funcs.insert(key, value);
    }

    pub fn variables(&self) -> std::collections::hash_map::Iter<K, V>
    {
        self.vars.iter()
    }

    pub fn functions(&self)
        -> std::collections::hash_map::Iter<K, (Vec<V>, ContextFunction<K, V>)>
    {
        self.funcs.iter()
    }
}

impl std::fmt::Display for Context<String, NodeBox>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        for (k, v) in self.vars.iter() {
            writeln!(f, "{}: {}", k, v);
        }
        for (k, (arg, v)) in self.funcs.iter() {
            let params = arg
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, x)| {
                    if 0 < i {
                        acc.push(',');
                    }
                    acc.push_str(&format!("{}", x));
                    acc
                });

            writeln!(f, "{}({}): {}", k, params, v);
        }
        Ok(())
    }
}

impl Default for Context<String, NodeBox>
{
    fn default() -> Self
    {
        use parser::parse;
        use program::execute_with_ctx;

        let mut new = Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        };

        // native functions
        {
            let ident1 = Box::new(Var("x".to_string()));
            let ident3 = Box::new(Var("base".to_string()));
            let closure = ContextFunction::Native(|ctx: &mut Self, args: &Vec<NodeBox>| {
                let base = super::execute_with_ctx(&args[0], ctx)?;
                let x = super::execute_with_ctx(&args[1], ctx)?;
                Ok(x.log(base))
            });
            new.setf(
                "log".to_string(),
                (vec![ident3.clone(), ident1.clone()], closure),
            );
        }

        // virtual functions
        {
            let mut extend_ctx = |expr: &str| {
                execute_with_ctx(&parse(expr).unwrap(), &mut new);
            };

            extend_ctx(format!("pi={}", std::f64::consts::PI).as_str());
            extend_ctx(format!("e={}", std::f64::consts::E).as_str());

            extend_ctx("log2(x)=log(2,x)");
            extend_ctx("ln(x)=log(e,x)");
            extend_ctx("sqrtn(n,x)=x^(1/n)");
            extend_ctx("sqrt(x)=sqrtn(2,x)");
        }

        new
    }
}
