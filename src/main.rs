fn main() {
    feature::main();
}

mod parser {

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
                Operation(op) => match op {
                    Add(x, y) => Ok(execute(x)? + execute(y)?),
                    Sub(x, y) => Ok(execute(x)? - execute(y)?),
                    Mul(x, y) => Ok(execute(x)? * execute(y)?),
                    Pow(x, y) => Ok(execute(x)?.powf(execute(y)?)),
                    Sqrt(x)   => Ok(execute(x)?.sqrt()),
                    Div(x, y) => {
                        let arg2 = execute(y)?;
                        if arg2 == 0 as Num {
                            Err("Division with 0")
                        } else {
                            Ok(execute(x)? / arg2)
                        }
                    },
                    _ => Err("Not implemented"),
                },
                Value(n) => Ok(*n),
            }
        }
    
    #[derive(Debug)]
    enum Token {
        Operator(u8, char),
        Number(String),
        /*
           Variable(&'static str),
           ParanOpen,
           ParanClose,
           */
    }

    #[derive(Debug)]
    enum ParseToken {
        Done(Node),
        Waiting(Token),
        /*
           Variable(&'static str),
           ParanOpen,
           ParanClose,
           */
    }
    
    fn adjust_binding(tokens: &mut Vec<ParseToken>, group: &mut Vec<usize>)
    {
        use self::Token::*;
        use self::ParseToken::*;
        //println!("{:?}", hbind_group);
        for i in group.iter().rev() {
            let n = i - 1;
            let prev = tokens.remove(n);
            let curr = tokens.remove(n);
            let next = tokens.remove(n);
            
            //println!("{:?} - {:?} - {:?}", prev, curr, next);

            let done = match (prev, curr, next) {
                //(Waiting(p), Waiting(c), Waiting(n)) => parse(p, c, n),
                (Done(d), Waiting(op), Done(w))
                    => Done(match (d, op, w) {
                        (op @ Operation(_), Operator(_, c), v1 @ Value(_)) => Operation(
                            match c {
                                '+' => Add(Box::new(op), Box::new(v1)), 
                                '-' => Sub(Box::new(op), Box::new(v1)), 
                                '*' => Mul(Box::new(op), Box::new(v1)), 
                                _   => Div(Box::new(op), Box::new(v1)), 
                            }
                        ),
                        (v1 @ Value(_), Operator(_, c), op @ Operation(_)) => Operation(
                            match c {
                                '+' => Add(Box::new(op), Box::new(v1)), 
                                '-' => Sub(Box::new(op), Box::new(v1)), 
                                '*' => Mul(Box::new(op), Box::new(v1)), 
                                _   => Div(Box::new(op), Box::new(v1)), 
                            }
                        ),
                        (v1 @ Value(_), Operator(_, c), v2 @ Value(_)) => Operation(
                            match c {
                                '+' => Add(Box::new(v1), Box::new(v2)), 
                                '-' => Sub(Box::new(v1), Box::new(v2)), 
                                '*' => Mul(Box::new(v1), Box::new(v2)), 
                                _   => Div(Box::new(v1), Box::new(v2)), 
                            }
                        ),
                        (op1 @ Operation(_), Operator(_, c), op2 @ Operation(_)) => Operation(
                            match c {
                                '+' => Add(Box::new(op1), Box::new(op2)), 
                                '-' => Sub(Box::new(op1), Box::new(op2)), 
                                '*' => Mul(Box::new(op1), Box::new(op2)), 
                                _   => Div(Box::new(op1), Box::new(op2)), 
                            }
                        ),
                        (_, _, _) => {
                            panic!("neeeej")
                        },
                    }),
                (_, _, _) => Done(Value(0.0)),
            }; 

            tokens.insert(n, done);
        }
        //println!("{:?}", tokens);
    }

    fn parse(prev: Token, curr: Token, next: Token)
        -> ParseToken
    {
        use self::Token::*;
        use self::Node;
        use self::Node::*;
        use self::ParseToken::*;

        Done(
            match (prev, curr, next) {
                (Number(arg1), Operator(_, op), Number(arg2)) => {
                    let arg1 = arg1.parse::<f64>().unwrap();
                    let arg2 = arg2.parse::<f64>().unwrap();

                    //println!("{} {}", arg1, arg2);

                    Operation(
                        match op {
                            '+' => Add(Box::new(Value(arg1)), Box::new(Value(arg2))), 
                            '-' => Sub(Box::new(Value(arg1)), Box::new(Value(arg2))), 
                            '*' => Mul(Box::new(Value(arg1)), Box::new(Value(arg2))), 
                            _   => Div(Box::new(Value(arg1)), Box::new(Value(arg2))), 
                        }
                    )
                },
                (_, _, _) => {
                    panic!("all must wait for now");
                    Value(0.0)
                },
            }
        )
    }

    fn tokenize(script: String)
        -> Vec<Token>
    {
        use self::Token::*;

        let mut buffer = String::new();
        let mut tokens: Vec<Token> = Vec::new();
        
        let copy = script.clone();

        for c in copy.split("") {

            // FIXME: what if two numbers are separated by space like `10 10`
            //        currently, the space is pushed and parsed

            match c {
                op @ "+" | op @ "-" | op @ "*" | op @ "/" => {
                    if !buffer.is_empty() {
                        tokens.push(Number(buffer.clone()));
                    }

                    let op = op.chars().next().unwrap();
                    if op == '+' || op == '-' {
                        tokens.push(Operator(1, op));
                    } else {
                        tokens.push(Operator(2, op));
                    }

                    buffer.clear();
                },
                c => {
                    buffer.push_str(c);
                }
            }
        }

        if !buffer.is_empty() {
            tokens.push(Number(buffer.clone()));
        }

        tokens
    }

    pub fn main()
    {
        use self::Token::*;
        use self::ParseToken::*;

        let script = String::from("2+10/2-2*1+1");
        
        let program = Node::Value(0.0);

        println!("Script: {}", script);

        let mut tokens: Vec<ParseToken> = tokenize(script)
                        .into_iter()
                        .map(|t| match t {
                            Number(arg) => {
                                let num = arg.parse::<f64>().unwrap();
                                Done(Value(num))
                            },
                            _ => Waiting(t),
                        })
                        .collect();

        println!("Tokens: {:?}", tokens);
        println!();
        
        'outer: loop {
            let mut hbind = 0;
            let mut hbind_min = 0;
            let mut hbind_group: Vec<usize> = Vec::new();

            {
                let iter = tokens.iter();

                for (i, token) in iter.enumerate() {
                    //println!("{} {:?}", i, token);

                    if let Waiting(token) = token {
                        match token {
                            Operator(binding, _) => {
                                if &hbind < binding {
                                    hbind_group.clear();
                                    hbind = *binding;
                                }
                                if &hbind == binding {
                                    hbind_group.push(i);
                                }
                                if binding < &hbind_min {
                                    hbind_min = *binding;
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }

            if hbind == hbind_min {
                break 'outer;
            }

            println!("pass");
            adjust_binding(&mut tokens, &mut hbind_group);
        }
        
        if let Done(prog) = tokens.into_iter().next().unwrap() {
            println!("{:?}", prog);
            println!("{:?}", execute(&prog));
        }
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
