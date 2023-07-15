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
    rc::Rc,
};

pub struct Lox {
    env: Rc<Environment>,
    stack: Vec<LoxValue>,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            env: Rc::new(Environment::new()),
            stack: vec![],
        }
    }

    pub fn exec(&mut self, source: &str) -> LoxResult {
        let ScanResult {
            tokens,
            errors: scan_errors,
        } = Scanner::scan(source);
        if !scan_errors.is_empty() {
            for err in scan_errors.iter() {
                error!("Error: {}", err.to_string());
            }
            return Err(LoxError::Runtime("Syntax errors encountered".into()));
        }
        let ParseResult {
            statements,
            errors: parse_errors,
        } = Parser::parse(tokens);
        if !parse_errors.is_empty() {
            for err in parse_errors.iter() {
                error!("Error: {}", err.to_string());
            }
            return Err(LoxError::Runtime("Syntax errors encountered".into()));
        }
        for stmt in statements.iter() {
            self.evaluate_stmt(&mut self.env.clone(), stmt)?;
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

    fn evaluate_stmt(&mut self, env: &mut Rc<Environment>, stmt: &Stmt) -> LoxResult {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate_expr(env, expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(env, expr)?;
                info!("{}", value.to_string());
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate_expr(env, expr)?,
                    None => LoxValue::Nil,
                };
                Rc::get_mut(env)
                    .unwrap()
                    .declare_by_token(name, value);
                Ok(())
            }
            Stmt::Block(statements) => {
                self.open_block();
                for stmt in statements.iter() {
                    if let Err(e) = self.evaluate_stmt(env, stmt) {
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
                let cond = self.evaluate_expr(env, condition)?;
                if cond.is_truthy() {
                    self.evaluate_stmt(env, body)
                } else if let Some(else_stmt) = else_branch {
                    self.evaluate_stmt(env, else_stmt)
                } else {
                    Ok(())
                }
            }
            Stmt::WhileLoop { condition, body } => {
                while self.evaluate_expr(env, condition)?.is_truthy() {
                    self.evaluate_stmt(env, body)?;
                }
                Ok(())
            }
            Stmt::Fun { name, params, body } => {
                let identifier = name.lexeme_str();
                let fun: LoxValue = LoxFunction {
                    name: Some(identifier.clone()),
                    params: params.clone(),
                    body: FunctionBody::Block(body.clone()),
                    closure: Some(Rc::new(Environment::inner(self.env.clone()))),
                }.into();
                Rc::get_mut(env).unwrap().declare(&identifier, fun);
                Ok(())
            }
            Stmt::Return(expr) => {
                let last = self.stack.len() - 1;
                self.stack[last] = self.evaluate_expr(env, expr)?;
                Ok(())
            }
        }
    }

    fn evaluate_expr(&mut self, env: &mut Rc<Environment>, expr: &Expr) -> LoxResult<LoxValue> {
        match expr {
            Expr::Literal(value) => Ok(LoxValue::from(value.clone())),
            Expr::Unary { operator, right } => match operator.kind {
                TokenKind::Bang => {
                    let right_value = self.evaluate_expr(env, right)?.is_truthy();
                    Ok(LoxValue::Boolean(!right_value))
                }
                _ => Err(LoxError::Runtime(format!(
                    "Unknown unary operator \"{}\"",
                    operator
                ))),
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => self.evaluate_binary_expr(env, operator, left, right),
            Expr::Grouping(inner) => self.evaluate_expr(env, inner),
            Expr::Identifier(name) => self.env.get_by_token(name),
            Expr::Assignment { name, value } => {
                let val = self.evaluate_expr(env, value)?;
                Rc::get_mut(&mut self.env)
                    .unwrap()
                    .assign_by_token(name, val.clone())?;
                Ok(val)
            }
            Expr::Logical {
                operator,
                left,
                right,
            } => match operator.kind {
                TokenKind::Or => {
                    let mut val = self.evaluate_expr(env, left)?;
                    if !val.is_truthy() {
                        val = self.evaluate_expr(env, right)?;
                    }
                    Ok(val)
                }
                TokenKind::And => {
                    let mut val = self.evaluate_expr(env, left)?;
                    if val.is_truthy() {
                        val = self.evaluate_expr(env, right)?;
                    }
                    Ok(val)
                }
                _ => Err(LoxError::Runtime(format!(
                    "Expected logical operator, got \"{}\"",
                    operator.lexeme_str()
                ))),
            },
            Expr::Call { callee, arguments } => {
                if let LoxValue::Function(func) = self.evaluate_expr(env, callee)?
                {
                    if arguments.len() != func.params.len() {
                        Err(LoxError::Runtime(format!(
                            "Expected {} arguments, but got {}",
                            func.params.len(),
                            arguments.len(),
                        )))
                    } else {
                        let mut args: Vec<LoxValue> = vec![];
                        for arg in arguments.iter() {
                            args.push(self.evaluate_expr(env, arg)?);
                        }
                        let mut env = func.closure
                            .map(|env| env.clone())
                            .unwrap_or(self.env.clone());
                        let return_value = match func.body {
                            FunctionBody::Block(statements) => {
                                for (i, arg) in args.drain(0..).enumerate() {
                                    Rc::get_mut(&mut env).unwrap().declare(&func.params[i].lexeme_str(), arg);
                                }
                                self.stack.push(LoxValue::Nil);
                                for stmt in statements.iter() {
                                    self.evaluate_stmt(&mut env, stmt)?;
                                    if matches!(stmt, Stmt::Return(_)) {
                                        break;
                                    }
                                }
                                self.stack.pop().unwrap()
                            }
                            FunctionBody::Native(func) => func(&mut env, args)?,
                        };
                        Ok(return_value)
                    }
                } else {
                    Err(LoxError::Runtime("Attempted to call non-function".into()))
                }
            }
        }
    }

    fn evaluate_binary_expr(
        &mut self,
        env: &mut Rc<Environment>,
        operator: &Token,
        left: &Expr,
        right: &Expr,
    ) -> LoxResult<LoxValue> {
        let left_value = self.evaluate_expr(env, left)?;
        let right_value = self.evaluate_expr(env, right)?;
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
                    Err(LoxError::Runtime(format!(
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
                    Err(LoxError::Runtime(format!(
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
                    Err(LoxError::Runtime(format!(
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
                    Err(LoxError::Runtime(format!(
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
                    Err(LoxError::Runtime(format!(
                        "Invalid operands {} <= {}",
                        left_value.to_string(),
                        right_value.to_string(),
                    )))
                }
            }
            TokenKind::EqualEqual => Ok(LoxValue::Boolean(left_value == right_value)),
            TokenKind::BangEqual => Ok(LoxValue::Boolean(left_value != right_value)),
            _ => Err(LoxError::Runtime(format!(
                "Unknown binary operator \"{}\"",
                operator
            ))),
        }
    }

    fn open_block(&mut self) {
        self.env = Rc::new(Environment::inner(self.env.clone()));
    }

    fn close_block(&mut self) -> LoxResult {
        self.env = Rc::get_mut(&mut self.env).unwrap().close()?;
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
            assert_ne!(entries[0].body, "nil");
        });
        Ok(())
    }

    #[test]
    fn function_definitions() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            fun greet(name) {
                fun greeting() {
                    return "Hello, " + name + "!";
                }
                print greeting();
            }
            fun get_name() {
                return "world";
            }
            greet(get_name());
        "#,
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].body, "Hello, world!");
        });
        Ok(())
    }

    #[test]
    fn function_closure() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(
            r#"
            fun make_counter() {
                var i = 0;
                fun count() {
                  i = i + 1;
                  print i;
                }
              
                return count;
              }
              
              var counter = make_counter();
              counter();
              counter();
            "#
        )?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "1");
            assert_eq!(entries[1].body, "2")
        });
        Ok(())
    }
}
