use super::node::{Node::*, NodeBox};
use program::{node::Identifier, ComputationResult, Num};
use std::collections::HashMap;

pub type Closure = fn(&mut Context, &Vec<NodeBox>) -> ComputationResult<Num>;

#[derive(Clone)]
pub enum ContextFunction
{
    Virtual(NodeBox),
    Native(Closure),
}

impl std::fmt::Debug for ContextFunction
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

impl std::fmt::Display for ContextFunction
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

#[derive(Clone, Debug)]
pub struct Context
{
    vars: HashMap<Identifier, NodeBox>,
    funcs: HashMap<Identifier, (Vec<NodeBox>, ContextFunction)>,
    deps: HashMap<Identifier, Option<Vec<Identifier>>>,
}

impl Context
{
    pub fn get(&self, key: &Identifier) -> Option<&NodeBox>
    {
        self.vars.get(key)
    }

    pub fn getf(&self, key: &Identifier) -> Option<&(Vec<NodeBox>, ContextFunction)>
    {
        self.funcs.get(key)
    }

    pub fn set(&mut self, key: Identifier, value: NodeBox) -> ComputationResult<()>
    {
        self.update_depdendencies(&key, &value)?;
        self.vars.insert(key, value);
        Ok(())
    }

    pub fn setf(
        &mut self,
        key: Identifier,
        value: (Vec<NodeBox>, ContextFunction),
    ) -> ComputationResult<()>
    {
        match value.1 {
            ContextFunction::Virtual(ref node) => self.update_depdendencies(&key, node)?,
            _ => {}
        }
        self.funcs.insert(key, value);
        Ok(())
    }

    pub fn variables(&self) -> std::collections::hash_map::Iter<Identifier, NodeBox>
    {
        self.vars.iter()
    }

    pub fn functions(
        &self,
    ) -> std::collections::hash_map::Iter<Identifier, (Vec<NodeBox>, ContextFunction)>
    {
        self.funcs.iter()
    }

    fn update_depdendencies(&mut self, key: &Identifier, node: &NodeBox) -> ComputationResult<()>
    {
        let dependencies = node.idents();

        if let Some(ref deps) = dependencies {
            if !self.resolve_dependencies(&key, deps) {
                return Err(format!("`{}` is already referenced in `{}`", key, node));
            }
        }

        self.deps.insert(key.clone(), dependencies);

        Ok(())
    }

    fn resolve_dependencies(&self, key: &Identifier, dependencies: &Vec<Identifier>) -> bool
    {
        for dname in dependencies {
            if dname == key {
                return false;
            }
            match self.deps.get(dname).clone() {
                Some(Some(dlist)) => {
                    if dlist.contains(&key) || !self.resolve_dependencies(&dname, &dlist) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }
}

impl std::fmt::Display for Context
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

impl Default for Context
{
    fn default() -> Self
    {
        use parser::parse;
        use program::execute_with_ctx;

        let mut new = Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            deps: HashMap::new(),
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
