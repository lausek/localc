use std::vec::IntoIter;
use std::iter::Peekable;

use self::Token::*;

pub type Tokens = Vec<Token>;

const SPECIAL_CHARS: &[char] = &['+', '-', '*', '/', '^', '(', ')', ' ', '[', ']', ',', ';', '='];

#[derive(Clone, Debug)]
pub enum Token {
    Operator(String),
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

    let push_buffer = |tokens: &mut Tokens, buffer: &mut String| {
        // FIXME: validate buffer here; filter variable names
        //      return Err("message") if buffer is incorrect
        tokens.push(
            if buffer.parse::<f64>().is_err() {
                Ident(buffer.clone())
            }
            else {
                Number(buffer.clone())
            }
        );
        buffer.clear();
        Ok(())
    };

    for c in script.chars() {
        if SPECIAL_CHARS.contains(&c) {
            if !buffer.is_empty() {
                push_buffer(&mut tokens, &mut buffer)?;
            }

            // FIXME: this doesn't look good
            match c {
                '(' | '[' => {
                    paren_stack.push(c);
                    tokens.push(Paren(c));
                },
                ')' | ']' => {
                    if let Some(popd) = paren_stack.pop() {
                        if (popd == '(' && c != ')') || (popd == '[' && c != ']') {
                            return Err("nesting is not correct");
                        }
                    }
                    else {
                        return Err("nesting is not correct");
                    }
                    tokens.push(Paren(c));
                },
                '+' | '-' |
                '*' | '/' |
                '^' | '=' => {
                    let mut raw = String::new();
                    raw.push(c);
                    tokens.push(Operator(raw));
                },
                ',' | ';' => tokens.push(Sep(c)),
                _ => continue,
            }
        }
        else {
            buffer.push(c);
        }
    }

    if !buffer.is_empty() {
        push_buffer(&mut tokens, &mut buffer)?;
    }

    if !paren_stack.is_empty() {
        Err("nesting is not correct")
    }
    else {
        Ok(tokens)
    }
}
