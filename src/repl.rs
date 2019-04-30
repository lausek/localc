use super::*;

pub type ReplResult = Result<Option<lovm::Value>, String>;

pub struct Repl {
    pub parser: ExprParser,
    pub runtime: Runtime,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            parser: ExprParser::new(),
            runtime: Runtime::new(),
        }
    }

    // TODO: load stdlib module later
    pub fn with_stdlib() -> Self {
        Self::new()
    }

    pub fn run(&mut self, raw: &str) -> ReplResult {
        let mut program = self.parser.parse(raw).unwrap();
        self.run_expr(&program)
    }

    pub fn run_expr(&mut self, expr: &Expr) -> ReplResult {
        match expr {
            Expr::Comp(Operator::Store, lhs, rhs) => match lhs {
                box Expr::Func(name, Some(params)) => self.runtime.store_var(),
                box Expr::Ref(name) | box Expr::Func(name, None) => self.runtime.store_var(),
                _ => Err("assignment not allowed".to_string()),
            },
            _ => {
                // flat expressions can be executed directly
                let code_object = compiler::compile(expr)?;
                self.runtime.run(&code_object)
            }
        }
    }

    pub fn repeat(&mut self) -> ReplResult {
        use std::io::BufRead;

        for line in std::io::stdin().lock().lines() {
            let script = line.unwrap();
            println!("{:?}", self.run(&script));
        }

        Ok(None)
    }
}
