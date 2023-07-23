mod basic;
#[cfg(test)]
mod test_scripts;

use crate::basic::{Lox, LoxError};
use std::env;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), LoxError> {
    simple_logger::init().unwrap();
    let mut lox = Lox::new();
    let args: Vec<String> = env::args().collect();
    lox.exec_file(&args[1])
}
