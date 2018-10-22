use std::vec::IntoIter;
use std::iter::Peekable;

use self::Token::*;

pub type Tokens = Vec<Token>;

#[derive(Clone, Debug)]
pub enum Token {
    Operator(char),
    Number(String),
    Paren(char),
    Ident(String),
    Sep(char),
}

pub fn validate(tokens: Tokens)
    -> Result<Tokens, &'static str>
{
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
                (Operator(_), Operator(_)) => {
                    return Err("two operators without numbers");
                },
                (_, _) => continue,
            }
        }
    }
    Ok(tokens)
}

pub fn take_till_match(iter: &mut Peekable<IntoIter<Token>>, tillc: char)
    -> Tokens
{
    let mut stack: Vec<char> = vec![];
    let mut buffer: Vec<Token> = vec![];

    stack.push(tillc);

    for t in iter {
        match t {
            Paren(paren) => if paren == '(' || paren == '[' {
                stack.push(paren);
                buffer.push(Paren(paren));
            }
            else if !stack.is_empty() {
                let last = stack.pop().unwrap();
                if stack.is_empty() {
                    assert!(last == tillc);
                    break;
                }
                buffer.push(Paren(paren));

            },
            t => buffer.push(t),
        }
    }

    buffer
}

pub fn tokenize(script: &str)
    -> Result<Tokens, &'static str>
{
    let mut buffer = String::new();
    let mut tokens: Tokens = Vec::new();

    let mut paren_stack: Vec<char> = Vec::new();

    for c in script.chars() {
        match c {
            '+' | '-' | '*' | '/' | '^' | '(' | ')' | ' ' | '[' | ']' | ',' | ';' | '=' => {
                if !buffer.is_empty() {
                    tokens.push(
                        if buffer.parse::<f64>().is_err() {
                            Ident(buffer.clone())
                        }
                        else {
                            Number(buffer.clone())
                        }
                    );
                    buffer.clear();
                }

                // FIXME: this doesn't look good

                let op = c;

                if op == ' ' {
                    continue;
                }

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
                        }
                        else {
                            return Err("nesting is not correct");
                        }
                        tokens.push(Paren(op));
                    },
                    op @ '+' | op @ '-' |
                    op @ '*' | op @ '/' |
                    op @ '^' | op @ '=' => tokens.push(Operator(op)),
                    op @ ',' | op @ ';' => tokens.push(Sep(op)),
                    _ => unreachable!(),
                }
            },
            c => {
                buffer.push(c);
            }
        }
    }

    if !buffer.is_empty() {
        tokens.push(
            if buffer.parse::<f64>().is_err() {
                Ident(buffer.clone())
            }
            else {
                Number(buffer.clone())
            }
        );
    }

    if !paren_stack.is_empty() {
        Err("nesting is not correct")
    }
    else {
        Ok(tokens)
    }
}
