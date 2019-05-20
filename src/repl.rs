use super::*;

pub type ReplResult = Result<Option<lovm::Value>, String>;

// include content of stdlib here
const STDLIB: &str = include_str!("./lib/stdlib.lc");

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
        use std::fs::File;
        use std::io::{Read, Write};

        let module = match File::open("/tmp/stdlib.lcc") {
            Ok(mut file) => {
                let mut buffer = vec![];
                file.read_to_end(&mut buffer).unwrap();
                // TODO: if this failes, compile again
                Module::deserialize(&buffer).expect("invalid code in stdlib")
            }
            _ => {
                let mut repl = Repl::new();
                let mut compiled =
                    File::create("/tmp/stdlib.lcc").expect("could not create stdlib file");

                // TODO: this should be a function call
                for line in STDLIB.lines().filter(|line| !line.is_empty()) {
                    repl.run(&line).unwrap();
                }

                let module = repl.runtime.module.build().expect("building stdlib failed");
                compiled.write_all(&module.serialize().unwrap()).unwrap();
                module
            }
        };

        let mut new = Self::new();
        new.runtime.vm.data.modules.load(&module).unwrap();
        new
    }

    pub fn run(&mut self, raw: &str) -> ReplResult {
        match &self.parser.parse(raw) {
            Ok(program) => self.run_expr(program),
            Err(err) => Err(format!("{:?}", err)),
        }
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
