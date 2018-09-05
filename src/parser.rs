use program::{Node, Node::*};
use self::{Token::*, ParseToken::*};

#[derive(Debug)]
enum ParseToken {
    Done(Node),
    Waiting(Token),
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

pub fn parse(script: String)
    -> Result<Node, &'static str>
{
    match make_parseable(tokenize(script)) {
        Ok(mut tokens) => {
            'outer: loop {
                let mut hbind = 0;
                let mut hbind_min = 0;
                let mut hbind_group: Vec<usize> = Vec::new();

                {
                    let iter = tokens.iter();

                    for (i, token) in iter.enumerate() {
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

                adjust_binding(&mut tokens, &mut hbind_group);
            }

            if let Done(prog) = tokens.into_iter().next().unwrap() {
                Ok(prog)
            } else {
                Err("jag vet inte")
            }
        },
        Err(msg) => Err(msg),
    }
}

fn make_parseable(tokens: Vec<Token>)
    -> Result<Vec<ParseToken>, &'static str>
{
    let mut ptokens: Vec<ParseToken> = Vec::new();
    for token in tokens {
        match token {
            Number(arg) => if let Ok(num) = arg.parse::<f64>() {
                ptokens.push(Done(Value(num)));
            } else {
                return Err("could not parse number")
            },
            _ => {
                ptokens.push(Waiting(token));
            },
        }
    }
    Ok(ptokens)
}

fn tokenize(script: String)
    -> Vec<Token>
{
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

fn adjust_binding(tokens: &mut Vec<ParseToken>, group: &mut Vec<usize>)
{
    // if we change the order of the tokens vec by removing 3 items, we
    // have to normalize the index access in the next cycle
    let mut normalize = 0;

    for i in group.iter() {
        let n = {
            let res = (i - 1).overflowing_sub(normalize);
            if res.1 {0} else {res.0}
        };

        let prev = tokens.remove(n);
        let curr = tokens.remove(n);
        let next = tokens.remove(n);

        normalize += 2;

        let done = match (prev, curr, next) {
            (Done(n1), Waiting(op), Done(n2)) => Done(match op {
                Operator(_, c) => match c {
                    '+' => Add(Box::new(n1), Box::new(n2)), 
                    '-' => Sub(Box::new(n1), Box::new(n2)), 
                    '*' => Mul(Box::new(n1), Box::new(n2)), 
                    _   => Div(Box::new(n1), Box::new(n2)), 
                },
                _ => panic!("neeeej"),
            }),
            (_, _, _) => panic!("neeeej"),
        }; 

        tokens.insert(n, done);
    }
}
