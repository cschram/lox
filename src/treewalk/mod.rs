mod error;
mod scanner;

pub use self::error::LoxError;
use self::scanner::{Scanner, ScanResult};
use std::{fs::File, io::{BufReader}};
use log::{info, error};

pub struct Lox;

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_file(&mut self, path: &str) -> Result<(), LoxError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut scanner = Scanner::from_buf(&mut reader);
        let ScanResult { tokens, errors } = scanner.scan();
        for err in errors.iter() {
            error!("Error: {}", err.to_string());
        }
        for token in tokens.iter() {
            info!("Token: {}", token.to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_run_file() {
        let mut lox = Lox::new();
        assert!(lox.run_file("doesntexist.lox").is_err());
    }
}
