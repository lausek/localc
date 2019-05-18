#![feature(box_patterns)]
#![feature(box_syntax)]
#![allow(clippy::all)]

extern crate env_logger;
#[macro_use]
extern crate lalrpop_util;
extern crate lazy_static;
extern crate log;
extern crate rand;
extern crate regex;

pub mod ast;
pub mod compiler;
pub mod repl;
pub mod runtime;
#[macro_use]
pub mod test;

lalrpop_mod!(pub expr);

use ast::*;
use expr::*;
use repl::*;
use runtime::*;

pub type Module = lovm::Module;
