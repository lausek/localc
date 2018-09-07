use program::{Node, Node::*};
use self::{Token::*, ParseToken::*};

#[derive(Debug)]
enum ParseToken {
    Done(Node),
    Waiting(Token),
}

#[derive(Clone, Debug)]
enum Token {
    Operator(u8, char),
    Number(String),
    Paren(char),
}

pub fn parse(script: String)
    -> Result<Node, &'static str>
{
    let tokens = reduce(validate(tokenize(script)?)?)?;
    match make_parseable(tokens) {
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
                                    if binding < &mut hbind_min {
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

fn validate(tokens: Vec<Token>)
    -> Result<Vec<Token>, &'static str>
{
    // FIXME: test parentheses nesting
    {
        let mut iter = tokens.iter().peekable();
        loop {
            let curr = iter.next();
            let next = iter.peek();
            
            if next.is_none() {
                break;
            }

            match (curr.unwrap(), next.unwrap()) {
                (Number(_), Number(_)) => {
                    return Err("two numbers with no operator");
                },
                (_, _) => continue,
            }
        }
    }
    Ok(tokens)
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

fn reduce(tokens: Vec<Token>)
    -> Result<Vec<Token>, &'static str>
{
    Ok(
        tokens
        .into_iter()
        .filter(|item| {
            match item {
                Paren(_) => false,
                _ => true,
            }
        })
        .collect()
    )
}

fn tokenize(script: String)
    -> Result<Vec<Token>, &'static str>
{
    let mut buffer = String::new();
    let mut tokens: Vec<Token> = Vec::new();
    let mut paren_level = 0;

    let copy = script.clone();

    for c in copy.split("") {
        match c {
            "+" | "-" | "*" | "/" | "(" | ")" | " " => {
                if !buffer.is_empty() {
                    tokens.push(Number(buffer.clone()));
                    buffer.clear();
                }

                let op = c.chars().next().unwrap();

                if op == ' ' {
                    continue;
                }

                let power = paren_level * 3;

                if op == '+' || op == '-' {
                    tokens.push(Operator(1 + power, op));
                }
                if op == '*' || op == '/' {
                    tokens.push(Operator(2 + power, op));
                }
                if op == '(' || op == ')' {
                    if op == '(' {
                        paren_level += 1;
                    } else {
                        paren_level -= 1;
                    }
                    tokens.push(Paren(op));
                }
            },
            c => {
                buffer.push_str(c);
            }
        }
    }

    if !buffer.is_empty() {
        tokens.push(Number(buffer.clone()));
    }

    if paren_level != 0 {
        Err("nesting is not correct")
    } else {
        Ok(tokens)
    }
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
