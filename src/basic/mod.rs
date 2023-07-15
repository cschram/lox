mod ast;
mod environment;
mod error;
mod globals;
mod parser;
mod scanner;
mod value;

pub use self::error::*;
use self::{ast::*, environment::*, parser::*, scanner::*, value::*};
use log::{error, info};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    mem::swap,
};

pub struct Lox {
    env: Box<Environment>,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            env: Box::new(Environment::new()),
        }
    }

    pub fn _get(&self, name: &str) -> LoxResult<LoxValue> {
        self.env.get(name)
    }

    pub fn _declare(&mut self, name: &str, value: LoxValue) {
        self.env.declare(name, value);
    }

    pub fn _assign(&mut self, name: &str, value: LoxValue) -> LoxResult<Option<LoxValue>> {
        self.env.assign(name, value)
    }

    pub fn exec(&mut self, source: &str) -> LoxResult {
        let ScanResult {
            tokens,
            errors: scan_errors,
        } = Scanner::scan(source);
        if scan_errors.len() > 0 {
            for err in scan_errors.iter() {
                error!("Error: {}", err.to_string());
            }
            return Err(LoxError::RuntimeError("Syntax errors encountered".into()));
        }
        let ParseResult {
            statements,
            errors: parse_errors,
        } = Parser::parse(tokens);
        if parse_errors.len() > 0 {
            for err in parse_errors.iter() {
                error!("Error: {}", err.to_string());
            }
            return Err(LoxError::RuntimeError("Syntax errors encountered".into()));
        }
        for stmt in statements.iter() {
            self.evaluate_stmt(&stmt)?;
        }
        Ok(())
    }

    pub fn _exec_file(&mut self, path: &str) -> LoxResult {
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

    fn evaluate_stmt(&mut self, stmt: &Stmt) -> LoxResult {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate_expr(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(expr)?;
                info!("{}", value.to_string());
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate_expr(expr)?,
                    None => LoxValue::Nil,
                };
                self.env.declare_by_token(name, value);
                Ok(())
            }
            Stmt::Block(statements) => {
                self.open_block();
                for stmt in statements.iter() {
                    if let Err(e) = self.evaluate_stmt(stmt) {
                        self.close_block()?;
                        return Err(e);
                    }
                }
                self.close_block()?;
                Ok(())
            }
            Stmt::IfElse {
                condition,
                body,
                else_branch,
            } => {
                let cond = self.evaluate_expr(condition)?;
                if cond.is_truthy() {
                    self.evaluate_stmt(body)
                } else if let Some(else_stmt) = else_branch {
                    self.evaluate_stmt(else_stmt)
                } else {
                    Ok(())
                }
            }
            Stmt::WhileLoop { condition, body } => {
                while self.evaluate_expr(condition)?.is_truthy() {
                    self.evaluate_stmt(body)?;
                }
                Ok(())
            }
        }
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> LoxResult<LoxValue> {
        match expr {
            Expr::Literal(value) => Ok(LoxValue::from(value.clone())),
            Expr::Unary { operator, right } => match operator.kind {
                TokenKind::Bang => {
                    let right_value = self.evaluate_expr(&right)?.is_truthy();
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
            } => self.evaluate_binary_expr(operator, left, right),
            Expr::Grouping(inner) => self.evaluate_expr(&inner),
            Expr::Identifier(name) => self.env.get_by_token(name),
            Expr::Assignment { name, value } => {
                let val = self.evaluate_expr(value)?;
                self.env.assign_by_token(name, val.clone())?;
                Ok(val)
            }
            Expr::Logical {
                operator,
                left,
                right,
            } => match operator.kind {
                TokenKind::Or => {
                    let mut val = self.evaluate_expr(left)?;
                    if !val.is_truthy() {
                        val = self.evaluate_expr(right)?;
                    }
                    Ok(val)
                }
                TokenKind::And => {
                    let mut val = self.evaluate_expr(left)?;
                    if val.is_truthy() {
                        val = self.evaluate_expr(right)?;
                    }
                    Ok(val)
                }
                _ => Err(LoxError::RuntimeError(format!(
                    "Expected logical operator, got \"{}\"",
                    operator.lexeme.as_ref().unwrap()
                ))),
            },
            Expr::Call { callee, arguments } => {
                if let LoxValue::Fun {
                    arity,
                    name: _,
                    fun,
                } = self.evaluate_expr(callee)?
                {
                    let mut args: Vec<LoxValue> = vec![];
                    for arg in arguments.iter() {
                        args.push(self.evaluate_expr(arg)?);
                    }
                    if args.len() != arity {
                        Err(LoxError::RuntimeError(format!(
                            "Expected {} arguments, but got {}",
                            arity,
                            args.len(),
                        )))
                    } else {
                        fun(args)
                    }
                } else {
                    Err(LoxError::RuntimeError(
                        "Attempted to call non-function".into(),
                    ))
                }
            }
        }
    }

    fn evaluate_binary_expr(
        &mut self,
        operator: &Token,
        left: &Box<Expr>,
        right: &Box<Expr>,
    ) -> LoxResult<LoxValue> {
        let left_value = self.evaluate_expr(&left)?;
        let right_value = self.evaluate_expr(&right)?;
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
                        left_value.get_number()? + right_value.get_number()?,
                    ))
                } else {
                    Err(LoxError::RuntimeError(format!(
                        "Invalid operands {} + {}",
                        left_value.to_string(),
                        right_value.to_string(),
                    )))
                }
            }
            TokenKind::Minus => Ok(LoxValue::Number(
                left_value.get_number()? - right_value.get_number()?,
            )),
            TokenKind::Star => Ok(LoxValue::Number(
                left_value.get_number()? * right_value.get_number()?,
            )),
            TokenKind::Slash => Ok(LoxValue::Number(
                left_value.get_number()? / right_value.get_number()?,
            )),
            TokenKind::Greater => {
                if left_value.is_number() && right_value.is_number() {
                    Ok(LoxValue::Boolean(
                        left_value.get_number()? > right_value.get_number()?,
                    ))
                } else {
                    Err(LoxError::RuntimeError(format!(
                        "Invalid operands {} > {}",
                        left_value.to_string(),
                        right_value.to_string(),
                    )))
                }
            }
            TokenKind::GreaterEqual => {
                if left_value.is_number() && right_value.is_number() {
                    Ok(LoxValue::Boolean(
                        left_value.get_number()? >= right_value.get_number()?,
                    ))
                } else {
                    Err(LoxError::RuntimeError(format!(
                        "Invalid operands {} >= {}",
                        left_value.to_string(),
                        right_value.to_string(),
                    )))
                }
            }
            TokenKind::Less => {
                if left_value.is_number() && right_value.is_number() {
                    Ok(LoxValue::Boolean(
                        left_value.get_number()? < right_value.get_number()?,
                    ))
                } else {
                    Err(LoxError::RuntimeError(format!(
                        "Invalid operands {} < {}",
                        left_value.to_string(),
                        right_value.to_string(),
                    )))
                }
            }
            TokenKind::LessEqual => {
                if left_value.is_number() && right_value.is_number() {
                    Ok(LoxValue::Boolean(
                        left_value.get_number()? <= right_value.get_number()?,
                    ))
                } else {
                    Err(LoxError::RuntimeError(format!(
                        "Invalid operands {} <= {}",
                        left_value.to_string(),
                        right_value.to_string(),
                    )))
                }
            }
            TokenKind::EqualEqual => Ok(LoxValue::Boolean(left_value == right_value)),
            TokenKind::BangEqual => Ok(LoxValue::Boolean(left_value != right_value)),
            _ => Err(LoxError::RuntimeError(format!(
                "Unknown binary operator \"{}\"",
                operator
            ))),
        }
    }

    fn open_block(&mut self) {
        let mut env = Box::new(Environment::new());
        swap(&mut self.env, &mut env);
        self.env = Box::new(Environment::inner(env));
    }

    fn close_block(&mut self) -> LoxResult {
        self.env = self.env.close()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mock_logger::MockLogger;

    #[test]
    fn print() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            var pi = 3.14;
            print pi;
            var foo;
            print foo;
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "3.14");
            assert_eq!(entries[1].body, "nil");
        });
        Ok(())
    }

    #[test]
    fn block_scope() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            var foo = "foo";
            {
                var foo = "bar";
                print foo;
            }
            print foo;
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "bar");
            assert_eq!(entries[1].body, "foo");
        });
        Ok(())
    }

    #[test]
    fn control_flow() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            if (true and (nil or "truthy")) {
                print "true";
            } else {
                print "false";
            }
            if (false) {
                print "false";
            } else {
                print "true";
            }
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "true");
            assert_eq!(entries[1].body, "true");
        });
        Ok(())
    }

    #[test]
    fn while_loop() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            var index = 4;
            while (index > 0) {
                print index;
                index = index - 1;
            }
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 4);
            assert_eq!(entries[0].body, "4");
            assert_eq!(entries[1].body, "3");
            assert_eq!(entries[2].body, "2");
            assert_eq!(entries[3].body, "1");
        });
        Ok(())
    }

    #[test]
    fn for_loop() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            var index = 42;
            for (var index = 0; index < 4; index = index + 1) {
                print index;
            }
            print index;
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 5);
            assert_eq!(entries[0].body, "0");
            assert_eq!(entries[1].body, "1");
            assert_eq!(entries[2].body, "2");
            assert_eq!(entries[3].body, "3");
            assert_eq!(entries[4].body, "42");
        });
        Ok(())
    }

    #[test]
    fn native_functions() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            print time();
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 1);
        });
        Ok(())
    }
}
