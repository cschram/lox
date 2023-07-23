mod basic;
mod test_scripts;

use crate::basic::{Lox, LoxError};

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), LoxError> {
    simple_logger::init().unwrap();

    let mut lox = Lox::new();
    lox.exec("1 + (3.5 / 1.2)")?;

    Ok(())
}
