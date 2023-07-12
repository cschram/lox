mod treewalk;

use crate::treewalk::{Lox, LoxError};

fn main() -> Result<(), LoxError> {
    simple_logger::init().unwrap();

    let mut lox = Lox::new();
    lox.exec("1 + (3.5 / 1.2)")?;

    Ok(())
}
