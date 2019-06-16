pub mod func;
pub mod overload;

pub use self::func::*;
pub use self::overload::*;

use super::*;

use lovm::*;

use std::collections::HashMap;

pub struct Runtime {
    fn_templates: HashMap<Name, Function>,
    pub unit: gen::UnitBuilder,
    pub(crate) vm: vm::Vm,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            fn_templates: HashMap::new(),
            unit: gen::UnitBuilder::new(),
            vm: vm::Vm::new(),
        }
    }

    pub fn store_var(&mut self, name: &Name, expr: &Expr) -> ReplResult {
        let value = self.run_expr(expr)?.unwrap();
        self.vm.data.globals.insert(name.clone(), value);
        Ok(None)
    }

    pub fn store_fun(&mut self, name: &Name, params: &TupleType, expr: &Expr) -> ReplResult {
        if !self.fn_templates.contains_key(name) {
            self.fn_templates.insert(name.clone(), Function::new());
        }

        let overload_co = compiler::compile_with_params_lazy(expr, params)?;
        let fn_template = self.fn_templates.get_mut(name).unwrap();
        fn_template.overload(params.clone(), overload_co);
        let co = fn_template.build().unwrap();

        self.unit.set(name, co);

        let unit = self.unit.build().unwrap();
        match self.vm.data.units.0.get_mut(0) {
            Some(slot) => *slot = unit,
            _ => self.vm.data.units.load(&unit)?,
        }

        Ok(None)
    }

    pub fn run_expr(&mut self, expr: &Expr) -> ReplResult {
        match expr {
            Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
                box Expr::Func(name, params) => self.store_fun(name, params, rhs),
                box Expr::Ref(name) => self.store_var(name, rhs),
                _ => Err("assignment not allowed".to_string()),
            },
            _ => {
                // TODO: if this returns a reference to an temporary object; drop it to save memory
                let co = compiler::compile_expr(expr)?;

                if cfg!(debug_assertions) {
                    println!("{:?}", co);
                }

                // no storage location given: execute directly
                self.run(co)
            }
        }
    }

    // thin wrapper for repl
    pub fn run(&mut self, co: CodeObject) -> ReplResult {
        let co = co.into_ref();

        self.vm.data.state = vm::VmState::Running;
        self.vm.run_object(co)?;
        let result = self.vm.data.vstack.pop();

        // clear vstack to avoid stack poisoning through invalid bytecode
        self.vm.data.vstack.clear();
        //assert_eq!(self.vm.data.vstack.drain(..).len(), 0);

        Ok(result)
    }
}
