use std::collections::HashMap;
use super::node::{Node::*, NodeBox};
use program::{ComputationResult, Num};

pub type Identifier     = String;
pub type GenericContext = Context<Identifier, NodeBox>;
pub type Closure<K,V>   = fn(&mut Context<K,V>, &Vec<V>) -> ComputationResult<Num>;

#[derive(Clone)]
pub enum ContextFunction<K, V> 
    where K: Eq + std::hash::Hash,
          V: std::fmt::Display
{
    Virtual(V),
    // FIXME: should return Result<V>
    Native(Closure<K, V>),
}

impl<K, V> std::fmt::Debug for ContextFunction<K, V>
    where K: Eq + std::hash::Hash,
          V: std::fmt::Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        use self::ContextFunction::*;
        match self {
            Virtual(n) => writeln!(f, "{}", n),
            _          => writeln!(f, "<native>"),
        }.unwrap();
        Ok(())
    }
}

impl<K, V> std::fmt::Display for ContextFunction<K, V>
    where K: Eq + std::hash::Hash,
          V: std::fmt::Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        use self::ContextFunction::*;
        match self {
            Virtual(n) => writeln!(f, "{}", n),
            _          => writeln!(f, "<native>"),
        }.unwrap();
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Context<K, V>
    where K: Eq + std::hash::Hash,
          V: std::fmt::Display
{
    vars: HashMap<K, V>,
    funcs: HashMap<K, (Vec<V>, ContextFunction<K, V>)>,
}

impl<K, V> Context<K, V> 
    where K: Eq + std::hash::Hash,
          V: std::fmt::Display
{
    pub fn get(&self, key: &K)
        -> Option<&V>
    {
        self.vars.get(key)
    }

    pub fn getf(&self, key: &K)
        -> Option<&(Vec<V>, ContextFunction<K, V>)>
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
}

impl std::fmt::Display for Context<String, NodeBox>
{
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        for (k, v) in self.vars.iter() {
            writeln!(f, "{}: {}", k, v);
        }
        for (k, (arg, v)) in self.funcs.iter() {
            let params = arg.iter()
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
        let mut new = Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        };

        new.set("pi".to_string(), Box::new(Val(std::f64::consts::PI)));
        new.set("e".to_string(), Box::new(Val(std::f64::consts::E)));

        let ident1 = Box::new(Var("x".to_string()));
        let _ident2 = Box::new(Var("y".to_string()));
        let ident3 = Box::new(Var("base".to_string()));

        {
            let closure = ContextFunction::Native(|ctx: &mut Self, args: &Vec<NodeBox>| {
                let base = super::execute_with_ctx(args.get(0).unwrap(), ctx)?;
                let x    = super::execute_with_ctx(args.get(1).unwrap(), ctx)?;
                Ok(x.log(base))
            });
            new.setf("log".to_string(), (vec![ident3.clone(), ident1.clone()], closure));
        }

        {
            let args = vec![Box::new(Val(2.0)), ident1.clone()];
            let log2 = ContextFunction::Virtual(Box::new(Func("log".to_string(), args)));
            new.setf("log2".to_string(), (vec![ident1.clone()], log2));
        }

        {
            let sqrt = ContextFunction::Virtual(Box::new(Pow(ident1.clone(), Box::new(Val(0.5)))));
            new.setf("sqrt".to_string(), (vec![ident1.clone()], sqrt));
        }

        new
    }
}
