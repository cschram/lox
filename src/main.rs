mod treewalk;

use crate::treewalk::{Lox, LoxError};

fn main() -> Result<(), LoxError> {
    simple_logger::init().unwrap();

    let mut lox = Lox::new();
    lox.run_file("scripts/helloworld.lox")?;
    
    Ok(())
}
