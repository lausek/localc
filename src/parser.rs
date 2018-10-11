use std::vec::IntoIter;

use self::{Token::*, TempToken::*};
use program::{Num, Node, Node::*};

#[derive(Debug)]
enum TempToken {
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

type Tokens = Vec<Token>;
type ParserResult<T> = Result<T, &'static str>;

pub fn parse(script: String)
    -> ParserResult<Node> 
{
    let tokens = tokenize(script)?;

    if tokens.len() == 0 {
        return Err("No expression given");
    }

    let valid_tokens = validate(tokens)?;
    parse_list(valid_tokens.into_iter())
}

fn parse_list(mut tokens: IntoIter<Token>)
    -> ParserResult<Node>
{
    let mut subcomps: Vec<TempToken> = Vec::new();

    loop {
        let t = tokens.next();
        
        if let Some(t) = t {

            match t {
                Paren(paren) => if paren == '(' || paren == '[' {
                    let subquery = take_till(&mut tokens, paren);
                    let node = parse_list(subquery.into_iter())?;
                    subcomps.push(Done(node));
                },
                Number(raw) => if let Ok(num) = raw.parse::<Num>() {
                    subcomps.push(Done(Value(num)));
                } else {
                    return Err("could not parse number")
                },
                node => subcomps.push(Waiting(node)),
            }

        } else {
            break;
        }
    }
    
    if subcomps.len() == 0 {
        return Err("No expression given");
    }

    // FIXME: implement ^-power operator here too

    reduce(&mut subcomps, &['*', '/']);

    reduce(&mut subcomps, &['+', '-']);

    if let Some(Done(node)) = subcomps.into_iter().next() {
        Ok(node)
    } else {
        panic!("subcomps contains more than one or no child after reduction.")
    }
}

fn take_till(iter: &mut IntoIter<Token>, tillc: char)
    -> Tokens
{
    // FIXME: should be a stack instead
    let mut lvl = 1;
    let mut stack: Vec<char> = Vec::new();
    let mut buffer: Vec<Token> = vec![];

    stack.push(tillc);

    while let Some(t) = iter.next() {
        match t {
            Paren(paren) => if paren == '(' || paren == '[' {
                stack.push(paren);
                buffer.push(Paren(paren));
            } else {
                if !stack.is_empty() {
                    let last = stack.pop().unwrap();
                    if stack.is_empty() {
                        assert!(last == tillc);
                        break;
                    }
                    buffer.push(Paren(paren));
                }
            },
            t => buffer.push(t),
        }
    }

    buffer
}

fn validate(tokens: Tokens)
    -> ParserResult<Tokens>
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

fn tokenize(script: String)
    -> ParserResult<Tokens>
{
    let mut buffer = String::new();
    let mut tokens: Tokens = Vec::new();

    let mut paren_stack: Vec<char> = Vec::new();

    let copy = script.clone();

    for c in copy.chars() {
        match c {
            '+' | '-' | '*' | '/' | '(' | ')' | ' ' | '[' | ']' | ',' | ';' => {
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

                let op = c;

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
                buffer.push(c);
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

fn reduce(tokens: &mut Vec<TempToken>, group: &[char])
{
    // if we change the order of the tokens vec by removing 3 items, we
    // have to normalize the index access in the next cycle
    let mut normalize = 0;

    let indices: Vec<usize> = tokens.iter()
                        .enumerate()
                        .filter(|t| {
                            if let Waiting(Operator(_, op)) = t.1 {
                                group.contains(op)
                            } else {
                                false
                            }
                        })
                        .map(|t| {
                            t.0 as usize
                        })
                        .collect();

    for i in indices.iter() {
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
            (x, y, z) => panic!("What is that? {:?}, {:?}, {:?}", x, y, z),
        }; 

        tokens.insert(n, done);
    }
}
