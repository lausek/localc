use super::*;

use lovm::gen::*;

use std::iter::Peekable;
use std::slice::Iter;

// localc's intermediate representation of a function. every function can be called with
// arguments that are variant over length and items.

#[derive(Debug)]
pub struct Function {
    overloads: Vec<(Overload, FunctionBuilder)>,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for (overload, fb) in self.overloads.iter() {
            writeln!(f, "{:?} => {}", overload, fb)?;
        }
        Ok(())
    }
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

    // generate a lovm-executable representation of the current function
    pub fn build(&self) -> Result<CodeObject, ()> {
        // every localc function takes an obligatory parameter for specifying the argument
        // amount that was meant to be passed
        let mut atable = FunctionBuilder::new().with_params(vec!["argc"]);
        let mut it = self.overloads.iter().peekable();

        while it.peek().is_some() {
            // build an atable (argument table) that contains overloads with the same argument
            // count `argc`. advances `it` until the argument count is different.
            let (argc, vtable) = build_vtable(&mut it);

            // check the passed argument count against the tables
            // expected count. branches to atable if equal
            atable.step(gen::Operation::cmp_eq().var("argc").op(argc).end());
            atable.branch_if(vtable);
        }

        atable.step(gen::Operation::ret());

        atable.build()
    }
}

// building the vtable works as follows:
// - take first argument (which is guaranteed to be Some(_) by caller) and extract argument count
// - compile dispatch entry using overload; merge first block onto `cases` (because all other
// checks will live in own blocks)
// - for all items in `it` having the same argc:
//      - compile dispatch entry using overload into function
//
//  add_case will work recursively by peeking if the next argument has the same argc. if so, it
//  takes that argument and immediately turns it into a function by calling add_case again. this
//  way all cases will be nested in another allowing for correct `jf` branching.

// if the atable jumps to such a block, we need to pop the desired argument count from the stack
fn build_vtable(it: &mut Peekable<Iter<(Overload, FunctionBuilder)>>) -> (usize, FunctionBuilder) {
    let first = it.next().unwrap();
    let argc = first.0.count();
    // generate default arguments in form `arg0, arg1, ... argn`
    let params = (0..argc).map(|i| format!("arg{}", i)).collect::<Vec<_>>();
    // default arguments are popped off the stack here
    let mut cases = FunctionBuilder::new().with_params(params);

    cases.branch(create_case(it, &first));

    // delimit vtable
    cases.step(gen::Operation::ret());

    (argc, cases)
}

fn create_case(
    it: &mut Peekable<Iter<(Overload, FunctionBuilder)>>,
    (overload, fb): &(Overload, FunctionBuilder),
) -> FunctionBuilder {
    let mut case = FunctionBuilder::new();
    let argc = overload.count();

    // not all values can be compared (e.g. variable idents), so
    // how many components really add up to the final condition?
    let mut comps = 0;
    for (i, param) in overload.iter().enumerate() {
        match param {
            // variable name in function declaration matches everything
            Expr::Ref(_) => {}
            Expr::Value(v) => {
                let name = format!("arg{}", i);
                case.step(gen::Operation::cmp_eq().var(name).op(v).end());
                if 0 < comps {
                    case.step(gen::Operation::and());
                }
                comps += 1;
            }
            // no other variants allowed in overload
            _ => unreachable!(),
        }
    }

    let args = overload
        .iter()
        .enumerate()
        .filter_map(|(i, param)| match param {
            Expr::Ref(_) => Some(format!("arg{}", i)),
            _ => None,
        })
        .collect::<Vec<_>>();

    // if the condition turned out to false, continue with next block
    if it.peek().as_ref().map_or(false, |(o, _)| argc == o.count()) {
        let next_arg = it.next().unwrap();
        let next_case = create_case(it, next_arg);
        case.branch_else(next_case);
    // if the current condition does not accept all arguments,
    // but is the last one with same argc just return
    } else if !overload.accepts_count(argc) {
        case.branch_else(vec![gen::Operation::ret()]);
    }

    // calling order does not need `.rev()`
    for arg in args.into_iter() {
        case.step(gen::Operation::push().var(arg).end());
    }

    // will not reach this place if condition was false
    case.branch(fb.clone());

    case
}
