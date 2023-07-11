mod ast;
mod error;
mod parser;
mod scanner;

pub use self::error::LoxError;
use self::{
    parser::{Parser},
    scanner::{ScanResult, Scanner},
};
use log::{error, info};
use std::{fs::File, io::{BufReader, BufRead}};

pub struct Lox;

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn exec(&mut self, source: &str) -> Result<(), LoxError> {
        let ScanResult { tokens, errors } = Scanner::scan(source);
        for err in errors.iter() {
            error!("Error: {}", err.to_string());
        }
        for token in tokens.iter() {
            info!("Token: {}", token.to_string());
        }
        let ast = Parser::parse(&tokens)?;
        Ok(())
    }

    pub fn exec_file(&mut self, path: &str) -> Result<(), LoxError> {
        let file = File::open(path)?;
        let source: String = BufReader::new(file)
            .lines()
            .flat_map(|l| {
                let mut line = l.unwrap().chars().collect::<Vec<char>>();
                line.push('\n');
                line
            })
            .collect();
        self.exec(&source)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_exec_file() {
        let mut lox = Lox::new();
        assert!(lox.exec_file("doesntexist.lox").is_err());
    }
}
