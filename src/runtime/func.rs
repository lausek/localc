use super::*;

use lovm::gen::*;

use std::iter::Peekable;
use std::slice::Iter;

// localc's intermediate representation of a function. every function can be called with
// arguments that are variant over length and items.

#[derive(Debug)]
pub struct Function {
    overloads: Vec<(Overload, CodeBuilder)>,
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

    pub fn overload<T>(&mut self, overload: T, fb: CodeBuilder)
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
        let mut atable = CodeBuilder::new().with_params(vec!["argc"]);
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

        atable.build(true)
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
fn build_vtable(it: &mut Peekable<Iter<(Overload, CodeBuilder)>>) -> (usize, CodeBuilder) {
    let first = it.next().unwrap();
    let argc = first.0.count();
    // generate default arguments in form `arg0, arg1, ... argn`
    let params = (0..argc).map(|i| format!("arg{}", i)).collect::<Vec<_>>();
    // default arguments are popped off the stack here
    let mut cases = CodeBuilder::new().with_params(params);

    cases.step(create_case(&first));

    // if the condition turned out to false, continue with next block
    while it.peek().as_ref().map_or(false, |(o, _)| argc == o.count()) {
        let next_arg = it.next().unwrap();
        let next_case = create_case(next_arg);
        cases.step(next_case);
    }

    (argc, cases)
}

fn create_case((overload, fb): &(Overload, CodeBuilder)) -> CodeBuilder {
    let mut case = CodeBuilder::new();

    // not all values can be compared (e.g. variable idents), so
    // how many components really add up to the final condition?
    let mut comps = 0;
    for (i, param) in overload.iter().enumerate() {
        match param {
            // variable name in function declaration matches everything
            Expr::Ref(_) => {}
            Expr::Value(v) => {
                let name = format!("arg{}", i);
                case.step(gen::Operation::cmp_eq().var(name).op(v.clone()).end());
                if 0 < comps {
                    case.step(gen::Operation::and());
                }
                comps += 1;
            }
            // no other variants allowed in overload
            _ => unreachable!(),
        }
    }

    // TODO: this push could be avoided somehow
    // in `match-everything` conditions we have to push a default value
    if comps == 0 {
        case.step(Operation::push().op(true).end());
    }

    let yes_branch = {
        let args = overload
            .iter()
            .enumerate()
            .filter_map(|(i, param)| match param {
                Expr::Ref(_) => Some(format!("arg{}", i)),
                _ => None,
            })
            .collect::<Vec<_>>();

        // this branch will be executed if the condition generated above is true
        let mut yes_branch = CodeBuilder::new();

        // add prelude for passing function arguments
        // calling order does not need `.rev()`
        for arg in args.into_iter() {
            yes_branch.step(gen::Operation::push().var(arg).end());
        }

        yes_branch.step(fb.clone());
        yes_branch.step(Operation::ret());

        yes_branch
    };

    case.branch_if(yes_branch);

    case
}
