use super::*;

use lovm::gen::*;
use lovm::*;

pub struct Runtime {
    module: ModuleBuilder,
    vm: vm::Vm,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            module: ModuleBuilder::new(),
            vm: vm::Vm::new(),
        }
    }

    pub fn store_var(&mut self, name: &Name, expr: &Expr) -> ReplResult {
        println!("storing var");
        let value = self.run_expr(expr)?.unwrap();
        self.vm.data.globals.insert(name.clone(), value);
        Ok(None)
    }

    pub fn store_fun(
        &mut self,
        name: &Name,
        params: &VmFunctionParameters,
        expr: &Expr,
    ) -> ReplResult {
        println!("storing function");
        let code_object = compiler::compile_args(expr, params)?;
        self.module.set(name, code_object);
        let module = self.module.build().unwrap();
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
                let code_object = compiler::compile(expr)?;
                self.run(&code_object)
            }
        }
    }

    // thin wrapper for repl
    pub fn run(&mut self, co: &CodeObject) -> ReplResult {
        self.vm.data.state = vm::VmState::Running;
        self.vm.run_object(co)?;
        Ok(self.vm.data.vstack.pop())
    }
}
