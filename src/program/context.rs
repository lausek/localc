use std::collections::HashMap;
use super::node::{Node::*, NodeBox};

pub type GenericContext = Context<String, NodeBox>;

#[derive(Clone, Debug, Default)]
pub struct Context<K, V>
    where K: Eq + std::hash::Hash
{
    vars: HashMap<K, V>,
    funcs: HashMap<K, (Vec<V>, V)>,
}

impl<K, V> Context<K, V> 
    where K: Eq + std::hash::Hash
{
    pub fn get(&self, key: &K)
        -> Option<&V>
    {
        self.vars.get(key)
    }

    pub fn getf(&self, key: &K)
        -> Option<&(Vec<V>, V)>
    {
        self.funcs.get(key)
    }

    pub fn set(&mut self, key: K, value: V)
    {
        self.vars.insert(key, value);
    }

    pub fn setf(&mut self, key: K, value: (Vec<V>, V))
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
        let mut standard = HashMap::new();

        standard.insert(String::from("pi"), Box::new(Value(std::f64::consts::PI)));
        standard.insert(String::from("e"), Box::new(Value(std::f64::consts::E)));

        Self {
            vars: standard,
            funcs: HashMap::new(),
        }
    }
}
