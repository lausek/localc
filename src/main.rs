fn main() {
    feature::main();
}

mod feature {
    use self::Node::*;
    use self::Operation::*;

    type Num = f64;
    type Res = Result<Num, &'static str>;

#[derive(Clone, Debug)]
    enum Operation {
        Add(Box<Node>, Box<Node>),
        Sub(Box<Node>, Box<Node>),
        Mul(Box<Node>, Box<Node>),
        Div(Box<Node>, Box<Node>),
        Pow(Box<Node>, Box<Node>),
        Sqrt(Box<Node>),
    }

#[derive(Clone, Debug)]
    enum Node {
        Operation(Operation),
        Value(f64),
    }

    fn execute(program: &Node)
        -> Res
        {

            match program {
                Operation(op) => {
                    match op {
                        Add(x, y) => Ok(execute(x)? + execute(y)?),
                        Sub(x, y) => Ok(execute(x)? - execute(y)?),
                        Mul(x, y) => Ok(execute(x)? * execute(y)?),
                        Pow(x, y) => Ok(execute(x)?.powf(execute(y)?)),
                        Sqrt(x)   => Ok(execute(x)?.sqrt()),
                        Div(x, y) => {
                            let arg1 = execute(x)?;
                            let arg2 = execute(y)?;
                            if arg2 == 0 as Num {
                                Err("Division with 0")
                            } else {
                                Ok(arg1 / arg2)
                            }
                        },
                        _ => Err("Not implemented"),
                    }
                },
                Value(n) => Ok(*n),
            }
        }

    pub fn main()
    {
        let program = Operation(Add(
                Box::new(Value(18.0)),
                Box::new(Value(18.0))
                ));

        let result = execute(&program);

        println!("Program: {:?}", program);
        println!("Result: {:?}", result);

    }
}

#[cfg(test)]
mod tests {
    use Node::*;
    use Operation::*;

    #[test]
    fn addition() {
        let program = Operation(Add(
                Box::new(Value(18.0)),
                Box::new(Value(18.0))
                ));
        assert_eq!(execute(&program).unwrap(), 36.0);
    }

}
