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

        println!("{:?}", atable);
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
    fn peek_same_argc(argc: usize, it: &mut Peekable<Iter<(Overload, FunctionBuilder)>>) -> bool {
        it.peek().as_ref().map_or(false, |(o, _)| argc == o.count())
    }

    let first = it.next().unwrap();
    let argc = first.0.count();
    // generate default arguments in form `arg0, arg1, ... argn`
    let params = (0..argc).map(|i| format!("arg{}", i)).collect::<Vec<_>>();
    // default arguments are popped off the stack here
    let mut cases = FunctionBuilder::new().with_params(params);

    let next_case_idx = match peek_same_argc(argc, it) {
        // TODO: this is wrong; should be the address of next comparison, not next branch!!
        true => Some(0),
        _ => None,
    };
    add_case(&mut cases, &first.0, &first.1, next_case_idx);

    while peek_same_argc(argc, it) {
        if let Some((overload, fb)) = it.next() {
            // if the overload satisfies all possible constraints (like a `default` branch),
            // we reached the end of our atable
            if overload.accepts_count(argc) {
                cases.branch(fb.clone());
                break;
            } else {
                let next_case_idx = match peek_same_argc(argc, it) {
                    // TODO: this is wrong; should be the address of next comparison, not next branch!!
                    true => Some(0),
                    _ => None,
                };
                add_case(&mut cases, &overload, &fb, next_case_idx);
            }
        }
    }

    // delimit vtable
    cases.step(gen::Operation::ret());

    (argc, cases)
}

fn add_case(
    vtable: &mut FunctionBuilder,
    overload: &Overload,
    fb: &FunctionBuilder,
    next: Option<usize>,
) {
    // not all values can be compared (e.g. variable idents), so
    // how many components really add up to the final condition?
    let mut comps = 0;
    for (i, param) in overload.iter().enumerate() {
        match param {
            // variable name in function declaration matches everything
            Expr::Ref(_) => {}
            Expr::Value(v) => {
                let name = format!("arg{}", i);
                vtable.step(gen::Operation::cmp_eq().var(name).op(0).end());
                if 0 < comps {
                    vtable.step(gen::Operation::and());
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
    if let Some(_) = next {
        // the offset of the next condition to check is:
        // - the amount of args as each arg causes a push instruction
        // - a jump instruction
        let next_cond = args.len() + 1;
        vtable.branch_else(next_cond);
        //vtable.step(Operation::jf().op(next_cond).end());
        // TODO: this should not jump to next branch but
        //atable.branch_else(next.clone());
    }

    for arg in args.into_iter() {
        vtable.step(gen::Operation::push().var(arg).end());
    }

    // will not reach this place if condition was false
    let mut branch_block = fb.clone();
    //branch_block.step(gen::Operation::ret());
    vtable.branch(branch_block);
}
