use super::*;

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
        let mut argc = 0;
        // the argument count table; takes several vtables
        let mut atable = FunctionBuilder::new();
        let mut vtable = FunctionBuilder::new();

        let mut it = self.overloads.iter();

        for (overload, fb) in self.overloads.iter() {
            if overload.count() != argc {
                func.step(gen::Operation::cmp().var("argc").op(argc).end());
                func.branch(gen::Operation::jeq(), atable.clone());

                atable = FunctionBuilder::new();
                argc = overload.count();
            }
            // TODO: append last item of iteration to atable
            // TODO: fill vtable
        }

        func.build()
    }
}
