use std::iter::Peekable;
use std::vec::IntoIter;

use regex::Regex;

use self::Token::*;

pub type Tokens = Vec<Token>;

const VALID_IDENT_REGEX: &str = r#"^[a-zA-Z][\w']*$"#;
const OPERATOR_CHARS: &[char] = &['+', '-', '*', '/', '^', '=', '<', '>', '!'];
const GRAMMAR_CHARS: &[char] = &['(', ')', '[', ']', ',', ';', ' '];

#[derive(Clone, Debug)]
pub enum Token
{
    Operator(String),
    Number(String),
    Paren(char),
    Ident(String),
    Sep(char),
}

// TODO: don't create Regex again every time; maybe use lazy_static?
pub fn is_valid_ident(seq: &String) -> bool
{
    !Regex::new(VALID_IDENT_REGEX).unwrap().is_match(seq)
}

pub fn is_special_char(c: &char) -> bool
{
    OPERATOR_CHARS.contains(c) || GRAMMAR_CHARS.contains(c)
}

// TODO: rename to optimize1 and merge +,- while validating
//       - other optimizations too?
pub fn validate(tokens: Tokens) -> Result<Tokens, &'static str>
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
                }
                (Operator(_), Operator(_)) => {
                    return Err("two operators without numbers");
                }
                (_, _) => continue,
            }
        }
    }
    Ok(tokens)
}

pub fn take_till_match(iter: &mut Peekable<IntoIter<Token>>, tillc: char) -> Tokens
{
    let mut stack: Vec<char> = vec![];
    let mut buffer: Vec<Token> = vec![];

    stack.push(tillc);

    for t in iter {
        match t {
            Paren(paren) => {
                if paren == '(' || paren == '[' {
                    stack.push(paren);
                    buffer.push(Paren(paren));
                } else if !stack.is_empty() {
                    let last = stack.pop().unwrap();
                    if stack.is_empty() {
                        assert!(last == tillc);
                        break;
                    }
                    buffer.push(Paren(paren));
                }
            }
            t => buffer.push(t),
        }
    }

    buffer
}

pub fn tokenize(script: &str) -> Result<Tokens, &'static str>
{
    let mut buffer = String::new();
    let mut tokens: Tokens = Vec::new();

    let mut paren_stack: Vec<char> = Vec::new();

    let push_buffer: fn(&mut Tokens, &mut String) -> Result<(), &'static str> =
        |tokens: &mut Tokens, buffer: &mut String| {
            if buffer.parse::<f64>().is_err() {
                if is_valid_ident(buffer) {
                    return Err("not a valid identifier");
                }
                tokens.push(Ident(buffer.clone()))
            } else {
                tokens.push(Number(buffer.clone()))
            }
            buffer.clear();
            Ok(())
        };

    let mut iter = script.chars().peekable();
    while let Some(c) = iter.next() {
        if is_special_char(&c) {
            if !buffer.is_empty() {
                push_buffer(&mut tokens, &mut buffer)?;
            }

            if OPERATOR_CHARS.contains(&c) {
                let mut raw = String::new();
                raw.push(c);
                take_operators(&mut raw, &mut iter);
                tokens.push(Operator(raw));
            } else {
                match c {
                    '(' | '[' => {
                        paren_stack.push(c);
                        tokens.push(Paren(c));
                    }
                    ')' | ']' => {
                        if let Some(popd) = paren_stack.pop() {
                            if (popd == '(' && c != ')') || (popd == '[' && c != ']') {
                                return Err("nesting is not correct");
                            }
                        } else {
                            return Err("nesting is not correct");
                        }
                        tokens.push(Paren(c));
                    }
                    '+' | '-' | '*' | '/' | '^' | '=' => {
                    }
                    ',' | ';' => tokens.push(Sep(c)),
                    _ => continue,
                }
            }

        } else {
            buffer.push(c);
        }
    }

    if !buffer.is_empty() {
        push_buffer(&mut tokens, &mut buffer)?;
    }

    if !paren_stack.is_empty() {
        Err("nesting is not correct")
    } else {
        Ok(tokens)
    }
}

fn take_operators(into: &mut String, iter: &mut Peekable<std::str::Chars>)
{
    loop {
        if iter.peek().is_none() {
            return;
        }
        if OPERATOR_CHARS.contains(iter.peek().unwrap()) {
            into.push(iter.next().unwrap());
        } else {
            break;
        }
    }
}
