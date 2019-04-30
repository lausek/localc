use super::*;

use lovm::*;

pub struct Runtime {
    vm: vm::Vm,
}

impl Runtime {
    pub fn new() -> Self {
        Self { vm: vm::Vm::new() }
    }

    pub fn store_var(&mut self) -> ReplResult {
        println!("storing var");
        Ok(None)
    }

    pub fn store_fun(&mut self) -> ReplResult {
        println!("storing function");
        Ok(None)
    }

    // thin wrapper for repl
    pub fn run(&mut self, co: &CodeObject) -> ReplResult {
        self.vm.data.state = vm::VmState::Running;
        self.vm.run_object(co)?;
        Ok(self.vm.data.vstack.pop())
    }
}
