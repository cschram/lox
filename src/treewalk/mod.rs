mod ast;
mod error;
mod parser;
mod scanner;
mod value;

pub use self::error::*;
use self::{
    ast::*,
    parser::Parser,
    scanner::{ScanResult, Scanner, TokenKind},
    value::*,
};
use log::error;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Lox;

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn exec(&mut self, source: &str) -> LoxResult<LoxValue> {
        let ScanResult { tokens, errors } = Scanner::scan(source);
        for err in errors.iter() {
            error!("Error: {}", err.to_string());
        }
        let ast = Parser::parse(&tokens)?;
        self.evaluate(&ast)
    }

    pub fn _exec_file(&mut self, path: &str) -> LoxResult<LoxValue> {
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

    fn evaluate(&mut self, expr: &Expr) -> LoxResult<LoxValue> {
        match expr {
            Expr::Literal { value } => {
                Ok(LoxValue::from(value.clone()))
            },
            Expr::Unary { operator, right } => match operator.kind {
                TokenKind::Bang => {
                    let right_value = self.evaluate(&right)?.is_truthy();
                    Ok(LoxValue::Boolean(!right_value))
                }
                _ => Err(LoxError::RuntimeError(format!(
                    "Unknown unary operator \"{}\"",
                    operator
                ))),
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left_value = self.evaluate(&left)?;
                let right_value = self.evaluate(&right)?;
                match operator.kind {
                    TokenKind::Plus => {
                        if left_value.is_string() || right_value.is_string() {
                            Ok(LoxValue::String(format!(
                                "{}{}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        } else if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Number(
                                left_value.get_number()? + right_value.get_number()?
                            ))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} + {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    TokenKind::Minus => {
                        Ok(LoxValue::Number(left_value.get_number()? - right_value.get_number()?))
                    },
                    TokenKind::Star => {
                        Ok(LoxValue::Number(left_value.get_number()? * right_value.get_number()?))
                    },
                    TokenKind::Slash => {
                        Ok(LoxValue::Number(left_value.get_number()? / right_value.get_number()?))
                    },
                    TokenKind::Greater => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(left_value.get_number()? > right_value.get_number()?))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} > {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    TokenKind::GreaterEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(left_value.get_number()? >= right_value.get_number()?))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} >= {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    TokenKind::Less => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(left_value.get_number()? < right_value.get_number()?))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} < {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    TokenKind::LessEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(left_value.get_number()? <= right_value.get_number()?))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} <= {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    TokenKind::EqualEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(left_value.get_number()? == right_value.get_number()?))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} == {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    TokenKind::BangEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(left_value.get_number()? != right_value.get_number()?))
                        } else {
                            Err(LoxError::RuntimeError(format!(
                                "Invalid operands {} != {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    },
                    _ => {
                        Err(LoxError::RuntimeError(format!(
                            "Unknown binary operator \"{}\"",
                            operator
                        )))
                    },
                }
            }
            Expr::Grouping { inner } => self.evaluate(&inner),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn literals() {
        let mut lox = Lox::new();
        assert!(lox.exec("nil").unwrap().is_nil());
        assert!(lox.exec("true").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("false").unwrap().get_boolean().unwrap());
        assert_eq!(lox.exec("3.14").unwrap().get_number().unwrap(), 3.14);
        assert_eq!(lox.exec("\"foo\"").unwrap().get_string().unwrap(), "foo");
    }

    #[test]
    fn arithmetic() {
        let mut lox = Lox::new();
        assert_eq!(lox.exec("(10 / 5) + (5 / 2) - (2 * 3)").unwrap().get_number().unwrap(), -1.5);
    }

    #[test]
    fn comparisons() {
        let mut lox = Lox::new();
        assert!(lox.exec("10 == 10").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("10 == 15").unwrap().get_boolean().unwrap());
        assert!(lox.exec("10 != 15").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("10 != 10").unwrap().get_boolean().unwrap());
        assert!(lox.exec("15 > 10").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("10 > 15").unwrap().get_boolean().unwrap());
        assert!(lox.exec("10 >= 10").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("10 >= 11").unwrap().get_boolean().unwrap());
        assert!(lox.exec("10 < 15").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("15 < 10").unwrap().get_boolean().unwrap());
        assert!(lox.exec("10 <= 10").unwrap().get_boolean().unwrap());
        assert!(!lox.exec("11 <= 10").unwrap().get_boolean().unwrap());
    }

    #[test]
    fn truthiness() {
        let mut lox = Lox::new();
        assert!(lox.exec("!false").unwrap().get_boolean().unwrap());
        assert!(lox.exec("!nil").unwrap().get_boolean().unwrap());
        assert!(!lox
            .exec("!\"hello world\"")
            .unwrap()
            .get_boolean()
            .unwrap());
        assert!(lox
            .exec("!!\"hello world\"")
            .unwrap()
            .get_boolean()
            .unwrap());
    }

    #[test]
    fn str_concat() {
        let mut lox = Lox::new();
        assert_eq!(lox.exec("\"foo\" + \"bar\"").unwrap().get_string().unwrap(), "foobar");
    }

    #[test]
    fn exec_file() {
        let mut lox = Lox::new();
        assert!(lox._exec_file("doesntexist.lox").is_err());
    }
}
