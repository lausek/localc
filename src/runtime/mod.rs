pub mod func;
pub mod overload;

pub use self::func::*;
pub use self::overload::*;

use super::*;

use lovm::*;

use std::collections::HashMap;

pub struct Runtime {
    fn_templates: HashMap<Name, Function>,
    pub module: gen::ModuleBuilder,
    pub(crate) vm: vm::Vm,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            fn_templates: HashMap::new(),
            module: gen::ModuleBuilder::new(),
            vm: vm::Vm::new(),
        }
    }

    pub fn store_var(&mut self, name: &Name, expr: &Expr) -> ReplResult {
        println!("storing var");
        let value = self.run_expr(expr)?.unwrap();
        self.vm.data.globals.insert(name.clone(), value);
        Ok(None)
    }

    pub fn store_fun(&mut self, name: &Name, params: &TupleType, expr: &Expr) -> ReplResult {
        println!("storing function");
        if !self.fn_templates.contains_key(name) {
            self.fn_templates.insert(name.clone(), Function::new());
        }

        let overload_co = compiler::compile_params_lazy(expr, params)?;
        let fn_template = self.fn_templates.get_mut(name).unwrap();
        fn_template.overload(params.clone(), overload_co);
        let co = fn_template.build().unwrap();

        self.module.set(name, co);

        println!("{}", fn_template);

        let module = self.module.build().unwrap();
        println!("{}", module);
        match self.vm.data.modules.0.get_mut(0) {
            Some(slot) => *slot = module,
            _ => self.vm.data.modules.load(&module)?,
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
                // no storage location given: execute directly
                self.run(&compiler::compile(expr)?)
            }
        }
    }

    // thin wrapper for repl
    pub fn run(&mut self, co: &CodeObject) -> ReplResult {
        self.vm.data.state = vm::VmState::Running;
        self.vm.run_object(co)?;
        let result = self.vm.data.vstack.pop();
        // clear vstack to avoid stack poisoning through invalid bytecode
        self.vm.data.vstack.clear();
        Ok(result)
    }
}
