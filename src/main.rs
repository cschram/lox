mod builtins;
mod class;
mod environment;
mod error;
mod expr;
mod function;
mod interpreter;
mod object;
mod parser;
mod resolver;
mod scanner;
mod state;
mod stmt;
mod value;

#[cfg(test)]
mod test_scripts;

use crate::{error::LoxResult, interpreter::LoxInterpreter};
use std::env;

fn main() -> LoxResult {
    simple_logger::init().unwrap();
    let mut lox = LoxInterpreter::new();
    let args: Vec<String> = env::args().collect();
    lox.exec_file(&args[1])
}
