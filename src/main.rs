fn main() {
    feature::main();
}

mod feature {
    use self::Node::*;
    use self::Operation::*;

    pub type Num = f64;
    pub type Res = Result<Num, &'static str>;

#[derive(Clone, Debug)]
    pub enum Operation {
        Add(Box<Node>, Box<Node>),
        Sub(Box<Node>, Box<Node>),
        Mul(Box<Node>, Box<Node>),
        Div(Box<Node>, Box<Node>),
        Pow(Box<Node>, Box<Node>),
        Sqrt(Box<Node>),
    }

#[derive(Clone, Debug)]
    pub enum Node {
        Operation(Operation),
        Value(f64),
    }

    pub fn execute(program: &Node)
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
    use feature::{*, Node::*, Operation::*};

    #[test]
    fn addition() {
        let program = Operation(Add(
                Box::new(Value(18.0)),
                Box::new(Value(18.0))
                ));
        assert_eq!(execute(&program).unwrap(), 36.0);
    }

    #[test]
    fn subtraction() {
        let program = Operation(Sub(
                Box::new(Value(18.0)),
                Box::new(Value(18.0))
                ));
        assert_eq!(execute(&program).unwrap(), 0.0);
    }

    #[test]
    fn multiplication() {
        let program = Operation(Mul(
                Box::new(Value(18.0)),
                Box::new(Value(18.0))
                ));
        assert_eq!(execute(&program).unwrap(), 324.0);
    }

    #[test]
    fn division() {
        let program = Operation(Div(
                Box::new(Value(18.0)),
                Box::new(Value(18.0))
                ));
        assert_eq!(execute(&program).unwrap(), 1.0);
    }

    #[test]
    fn division_zero() {
        let program = Operation(Div(
                Box::new(Value(18.0)),
                Box::new(Value(0.0))
                ));
        assert!(execute(&program).is_err(), "division with zero should not be possible");
    }
}
