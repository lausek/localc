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
        self.runtime.run_expr(expr)
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
