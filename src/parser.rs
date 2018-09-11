use program::{Num, Node, Node::*};
use self::{Token::*, ParseToken::*};

#[derive(Debug)]
enum ParseToken {
    Done(Node),
    Waiting(Token),
}

#[derive(Clone, Debug)]
enum Token {
    Operator(i8, char),
    Number(String),
    Paren(char),
    Ident(String),
    Sep(char),
}

type ParserResult<T> = Result<T, &'static str>;

pub fn parse(script: String)
    -> ParserResult<Node> 
{
    let tokens = reduce(validate(tokenize(script)?)?)?;
    match make_parseable(tokens) {
        Ok(tokens) => translate(tokens),
        Err(msg) => Err(msg),
    }
}

fn translate(mut tokens: Vec<ParseToken>)
    -> ParserResult<Node> 
{
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

    if let Some(tok) = tokens.into_iter().next() {
        if let Done(prog) = tok {
            Ok(prog)
        } else {
            Err("program couldn't be parsed")
        }
    } else {
        // program is empty
        Ok(Value(0.0))
    }
}

fn validate(tokens: Vec<Token>)
    -> ParserResult<Vec<Token>>
{
    {
        let mut iter = tokens.iter().peekable();
        // FIXME: try rewriting with while let 
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
    -> ParserResult<Vec<ParseToken>>
{
    let mut ptokens: Vec<ParseToken> = Vec::new();
    for token in tokens {
        match token {
            Number(arg) => if let Ok(num) = arg.parse::<Num>() {
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
    -> ParserResult<Vec<Token>>
{
    println!("{:?}", tokens);
    let mut next_tokens: Vec<Token> = Vec::new();
    let mut it = tokens.into_iter();

    while let Some(item) = it.next() {
        match item {
            Ident(name) => match name.as_str() {
                "sqrt" => {
                    /*
                    if let Some(Paren('(')) = it.next() {
                        let mut parens: Vec<Token> = vec![Paren('(')]; 
                        let term = it.take_while(move |t| {
                            false
                        });
                    } else {
                        panic!("sqrt is a function and must be continued by ()");
                    }
                    //it.take_while()
                    */
                },
                _ => {},
            },
            Paren(_) => {},
            _ => {
                next_tokens.push(item);
            },
        }
    }

    Ok(next_tokens)
}

fn tokenize(script: String)
    -> ParserResult<Vec<Token>>
{
    let mut buffer = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    let mut paren_stack: Vec<char> = Vec::new();

    let copy = script.clone();

    for c in copy.split("") {
        match c {
            "+" | "-" | "*" | "/" | "(" | ")" | " " | "[" | "]" | "," | ";" => {
                if !buffer.is_empty() {
                    tokens.push(
                        if buffer.parse::<f64>().is_err() {
                            Ident(buffer.clone())
                        } else {
                            Number(buffer.clone())
                        }
                    );
                    buffer.clear();
                }

                let op = c.chars().next().unwrap();

                if op == ' ' {
                    continue;
                }

                let power = paren_stack.len() as i8 * 3;

                match op {
                    op @ '(' | op @ '[' => {
                        paren_stack.push(op);
                        tokens.push(Paren(op));
                    },
                    op @ ')' | op @ ']' => {
                        if let Some(popd) = paren_stack.pop() {
                            if (popd == '(' && op != ')') || (popd == '[' && op != ']') {
                                return Err("nesting is not correct");
                            }
                        } else {
                            return Err("nesting is not correct");
                        }
                        tokens.push(Paren(op));
                    },
                    op @ '+' | op @ '-' => {
                        tokens.push(Operator(1 + power, op));
                    },
                    op @ '*' | op @ '/' => {
                        tokens.push(Operator(2 + power, op));
                    },
                    op @ ',' | op @ ';' => {
                        tokens.push(Sep(op));
                    },
                    _ => {},
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

    if paren_stack.len() != 0 {
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
