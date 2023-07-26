use std::fmt::Display;

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct SyntaxError {
    message: String,
    line: u32,
}

impl SyntaxError {
    pub fn new(message: String, line: u32) -> Self {
        Self { message, line }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Syntax error on line {}: {}", self.line, self.message)
    }
}

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("System Time Error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("{0}")]
    Syntax(SyntaxError),
    #[error("Resolution Error: {0}")]
    Resolution(String),
    #[error("Runtime Error: {0}")]
    Runtime(String),
}

pub type LoxResult<T = ()> = Result<T, LoxError>;
