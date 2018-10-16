pub mod lexer;

use std::vec::IntoIter;
use std::iter::Peekable;

use self::{TempToken::*, lexer::{Tokens, Token, Token::*}};
use program::{Num, node::{Node, Node::*, NodeBox}};

#[derive(Debug)]
enum TempToken {
    Done(Node),
    Wait(Token),
}

pub fn parse(script: String)
    -> Result<Node, &'static str> 
{
    let tokens = lexer::tokenize(script)?;

    if tokens.is_empty() {
        return Err("no expression given");
    }

    let valid_tokens = lexer::validate(tokens)?;
    parse_list(valid_tokens.into_iter().peekable())
}

fn parse_list(mut tokens: Peekable<IntoIter<Token>>)
    -> Result<Node, &'static str>
{
    let mut subcomps: Vec<TempToken> = Vec::new();

    while let Some(t) = tokens.next() {
        match t {
            Paren(paren) => if paren == '(' || paren == '[' {
                let subquery = lexer::take_till_match(&mut tokens, paren);
                let node = parse_list(subquery.into_iter().peekable())?;
                subcomps.push(Done(node));
            },
            Number(raw) => if let Ok(num) = raw.parse::<Num>() {
                subcomps.push(Done(Val(num)));
            } else {
                return Err("could not parse number")
            },
            Ident(ref name) => {
                if let Some(Paren('(')) = tokens.peek() {
                    tokens.next();
                    let args = parse_func_args(&mut tokens)?;
                    subcomps.push(Done(Func(name.clone(), args)));
                } else {
                    subcomps.push(Done(Var(name.to_string())));
                }
            },
            node => subcomps.push(Wait(node)),
        }
    }
    
    if subcomps.is_empty() {
        return Err("no expression given");
    }
    
    reduce(&mut subcomps, &['^']);

    reduce(&mut subcomps, &['*', '/']);

    reduce(&mut subcomps, &['+', '-']);

    reduce(&mut subcomps, &['=']);

    if let Some(Done(node)) = subcomps.into_iter().next() {
        Ok(node)
    } else {
        panic!("subcomps contains more than one or no child after reduction.")
    }
}

fn parse_func_args(iter: &mut Peekable<IntoIter<Token>>)
    -> Result<Vec<NodeBox>, &'static str>
{
    let subquery = lexer::take_till_match(iter, '(');
    let arguments = split_arguments(subquery)
                        .into_iter()
                        .map(|s| parse_list(s.into_iter().peekable()))
                        .collect::<Vec<Result<Node, &'static str>>>();

    if arguments.iter().any(|arg| arg.is_err()) {
        // FIXME: this should be translated into a more efficient approach
        for arg in arguments.iter() {
            if let Err(msg) = arg {
                return Err(msg);
            }
        }
    }

    Ok(arguments
        .into_iter()
        .map(|arg| Box::new(arg.unwrap()))
        .collect())
}

fn split_arguments(subquery: Vec<Token>)
    -> Vec<Tokens> 
{
    let mut stack: Vec<char> = vec![];
    let mut args: Vec<Tokens> = vec![];
    let mut buffer: Tokens = vec![];

    for token in subquery {
        match token {
            Paren(c) => {
                if c == '(' || c == '[' {
                    stack.push(c);
                }
                if c == ')' || c == ']' {
                    let last = stack.pop();
                    assert!(last.is_some());
                }
            },
            // FIXME: should both separators stay allowed?
            Sep(';') | Sep(',') => {
                // if the stack level is not 0, we need to 
                // push the buffer
                if stack.is_empty() {
                    args.push(buffer.clone());
                    buffer.clear();
                    continue;
                }
            },
            _ => {},
        }
        buffer.push(token);
    }

    if !buffer.is_empty() {
        args.push(buffer);
    }

    // stack must be empty at the end of this function
    assert!(stack.is_empty());

    args
}

fn reduce(tokens: &mut Vec<TempToken>, group: &[char])
{
    // if we change the order of the tokens vec by removing 3 items, we
    // have to normalize the index access in the next cycle
    let mut normalize = 0;

    // FIXME: this needs some refactoring

    let indices: Vec<usize> = tokens.iter()
                        .enumerate()
                        .filter(|t| {
                            if let Wait(Operator(op)) = t.1 {
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
            (Done(n1), Wait(op), Done(n2)) => Done(match op {
                Operator(c) => match c {
                    '+' => Add(Box::new(n1), Box::new(n2)), 
                    '-' => Sub(Box::new(n1), Box::new(n2)), 
                    '*' => Mul(Box::new(n1), Box::new(n2)), 
                    '/' => Div(Box::new(n1), Box::new(n2)), 
                    '^' => Pow(Box::new(n1), Box::new(n2)), 
                    '=' => Equ(Box::new(n1), Box::new(n2)), 
                    _   => unreachable!(), 
                },
                _ => panic!("neeeej"),
            }),
            (x, y, z) => panic!("What is that? {:?}, {:?}, {:?}", x, y, z),
        }; 

        tokens.insert(n, done);
    }
}
