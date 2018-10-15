use std::collections::HashMap;
use super::node::{Node::*, NodeBox};

pub type GenericContext = Context<String, NodeBox>;

#[derive(Debug, Default)]
pub struct Context<K, V>
    where K: Eq + std::hash::Hash
{
    pool: HashMap<K, V>,
}

impl<K, V> Context<K, V> 
    where K: Eq + std::hash::Hash
{

    pub fn get(&self, key: &K)
        -> Option<&V>
    {
        self.pool.get(key)
    }

    pub fn set(&mut self, key: K, value: V)
    {
        self.pool.insert(key, value);
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
            pool: standard,
        }
    }

}
