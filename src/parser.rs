use std::vec::IntoIter;
use std::iter::Peekable;

use self::TempToken::*;
use program::{Num, Node, Node::*};
use lexer::{*, Token::*};

#[derive(Debug)]
enum TempToken {
    Done(Node),
    Waiting(Token),
}

pub fn parse(script: String)
    -> Result<Node, &'static str> 
{
    let tokens = tokenize(script)?;

    if tokens.is_empty() {
        return Err("no expression given");
    }

    let valid_tokens = validate(tokens)?;
    parse_list(valid_tokens.into_iter().peekable())
}

fn parse_list(mut tokens: Peekable<IntoIter<Token>>)
    -> Result<Node, &'static str>
{
    let mut subcomps: Vec<TempToken> = Vec::new();

    while let Some(t) = tokens.next() {
        match t {
            Paren(paren) => if paren == '(' || paren == '[' {
                let subquery = take_till_match(&mut tokens, paren);
                let node = parse_list(subquery.into_iter().peekable())?;
                subcomps.push(Done(node));
            },
            Number(raw) => if let Ok(num) = raw.parse::<Num>() {
                subcomps.push(Done(Value(num)));
            } else {
                return Err("could not parse number")
            },
            Ident(ref name) => {
                if let Some(Paren('(')) = tokens.peek() {
                    tokens.next();
                    let func = parse_function(&mut tokens);
                    subcomps.push(Done(func));
                } else {
                    subcomps.push(Done(Var(name.to_string())));
                }
            },
            node => subcomps.push(Waiting(node)),
        }
    }
    
    if subcomps.is_empty() {
        return Err("no expression given");
    }
    
    reduce(&mut subcomps, &['^']);

    reduce(&mut subcomps, &['*', '/']);

    reduce(&mut subcomps, &['+', '-']);

    if let Some(Done(node)) = subcomps.into_iter().next() {
        Ok(node)
    } else {
        panic!("subcomps contains more than one or no child after reduction.")
    }
}

fn parse_function(iter: &mut Peekable<IntoIter<Token>>)
    -> Node
{
    let subquery = take_till_match(iter, '(');

    // TODO: split subquery correctly at separators (`;`)
    //       this must pay attention to nested expressions

    let node = parse_list(subquery.into_iter().peekable());
    // FIXME: this is just a temporary solution
    Func(Box::new(Value(1.0)))
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
                            if let Waiting(Operator(op)) = t.1 {
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
                Operator(c) => match c {
                    '+' => Add(Box::new(n1), Box::new(n2)), 
                    '-' => Sub(Box::new(n1), Box::new(n2)), 
                    '*' => Mul(Box::new(n1), Box::new(n2)), 
                    '/' => Div(Box::new(n1), Box::new(n2)), 
                    '^' => Pow(Box::new(n1), Box::new(n2)), 
                    _   => unreachable!(), 
                },
                _ => panic!("neeeej"),
            }),
            (x, y, z) => panic!("What is that? {:?}, {:?}, {:?}", x, y, z),
        }; 

        tokens.insert(n, done);
    }
}
