use localc::vm::Vm;

pub fn main()
{
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    let parser = localc::query::ExprParser::new();
    let mut vm = Vm::with_stdlib();

    for line in stdin.lock().lines() {
        let script = line.unwrap();
        let program = parser.parse(script.as_ref()).expect("parsing failed");
        println!("program: {:?}", program);
        let result = vm.run(&program);
        println!("result: {:?}", result);
    }
}
