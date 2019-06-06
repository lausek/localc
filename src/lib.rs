#![feature(box_patterns)]
#![feature(box_syntax)]
#![allow(clippy::all)]

extern crate env_logger;
extern crate lazy_static;
extern crate log;
extern crate rand;
extern crate regex;

pub mod repl;
pub mod runtime;
#[macro_use]
pub mod test;

use localc_cc_lib::{ast::*, compiler, expr::*};
use repl::*;
use runtime::*;

pub type Unit = lovm::Unit;
