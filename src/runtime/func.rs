use super::*;

use std::iter::Peekable;
use std::slice::Iter;

// localc's intermediate representation of a function. every function can be called with
// arguments that are variant over length and items.

pub struct Function {
    overloads: Vec<(Overload, FunctionBuilder)>,
}

impl Function {
    pub fn new() -> Self {
        Self { overloads: vec![] }
    }

    pub fn overload<T>(&mut self, overload: T, fb: FunctionBuilder)
    where
        T: Into<Overload>,
    {
        let overload = overload.into();
        match self
            .overloads
            .binary_search_by_key(&&overload, |item| &item.0)
        {
            Ok(idx) => self.overloads.get_mut(idx).unwrap().1 = fb,
            Err(idx) => self.overloads.insert(idx, (overload, fb)),
        }
    }

    pub fn build(&self) -> Result<gen::Function, ()> {
        println!("{:?}", self.overloads);
        let mut func = FunctionBuilder::new().with_params(vec!["argc"]);
        // the argument count table; takes several vtables
        //let mut atable = FunctionBuilder::new();
        //let mut vtable = FunctionBuilder::new();

        let mut it = self.overloads.iter().peekable();

        while it.peek().is_some() {
            let (argc, atable) = build_atable(&mut it);

            for i in 0..argc {
                let name = format!("arg{}", i);
                func.step(gen::Operation::pop().var(name).end());
            }

            func.step(gen::Operation::cmp().var("argc").op(argc).end());
            func.branch(gen::Operation::jeq(), atable);
        }

        /*
        for (overload, fb) in self.overloads.iter() {
            if overload.count() != argc {
                func.step(gen::Operation::cmp().var("argc").op(argc).end());
                func.branch(gen::Operation::jeq(), atable.clone());

                atable = FunctionBuilder::new();
                argc = overload.count();
            }

            atable.branch(gen::Operation::jeq(), vtable.clone());
            // TODO: append last item of iteration to atable
            // TODO: fill vtable
        }

        func.step(gen::Operation::cmp().var("argc").op(argc).end());
        func.branch(gen::Operation::jeq(), atable.clone());
        */

        func.build()
    }
}

fn build_atable(it: &mut Peekable<Iter<(Overload, FunctionBuilder)>>) -> (usize, FunctionBuilder) {
    let mut atable = FunctionBuilder::new();
    let first = it.next().unwrap();
    let argc = first.0.count();

    add_vtable(&mut atable, &first.0, &first.1);

    for (overload, fb) in it.take_while(|(o, _)| argc == o.count()) {
        add_vtable(&mut atable, &overload, &fb);
    }

    (argc, atable)
}

fn add_vtable(atable: &mut FunctionBuilder, overload: &Overload, fb: &FunctionBuilder) {
    for (i, param) in overload.iter().enumerate() {
        match param {
            // variable name in function declaration matches everything
            Expr::Ref(_) => {}
            Expr::Value(v) => {
                let name = format!("arg{}", i);
                atable.step(gen::Operation::cmp().var(name).op(0).end());
            }
            // no other variants allowed in overload
            _ => unreachable!(),
        }
    }
}
