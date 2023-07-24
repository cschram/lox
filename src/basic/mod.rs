mod ast;
mod builtins;
mod environment;
mod error;
mod parser;
mod resolver;
mod scanner;
mod value;

pub use self::error::*;
use self::{ast::*, environment::*, parser::*, resolver::*, scanner::*, value::*};
use log::{error, info};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Lox {
    env: Environment,
    stack: Vec<LoxValue>,
    locals: Locals,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            stack: vec![],
            locals: HashMap::new(),
        }
    }

    pub fn exec(&mut self, source: &str) -> LoxResult {
        let ParseResult {
            statements,
            errors: parse_errors,
        } = parse(source);
        if !parse_errors.is_empty() {
            for err in parse_errors.iter() {
                error!("Parse Error: {}", err.to_string());
            }
            return Err(LoxError::Runtime("Syntax errors encountered".into()));
        }
        for (key, value) in Resolver::bind(&statements)?.drain() {
            self.locals.insert(key, value);
        }
        for stmt in statements.iter() {
            self.evaluate_stmt(GLOBAL_SCOPE, stmt)?;
        }
        Ok(())
    }

    pub fn exec_file(&mut self, path: &str) -> LoxResult {
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

    fn evaluate_stmt(&mut self, scope: ScopeHandle, stmt: &Stmt) -> LoxResult {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate_expr(scope, expr)?;
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(scope, expr)?;
                info!("{}", value.to_string());
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate_expr(scope, expr)?,
                    None => LoxValue::Nil,
                };
                self.env.declare(Some(scope), name.lexeme_str(), value);
            }
            Stmt::Block(statements) => {
                let block_scope = self.env.new_scope(Some(scope));
                for stmt in statements.iter() {
                    self.evaluate_stmt(block_scope, stmt)?;
                }
            }
            Stmt::IfElse {
                condition,
                body,
                else_branch,
            } => {
                let cond = self.evaluate_expr(scope, condition)?;
                if cond.is_truthy() {
                    self.evaluate_stmt(scope, body)?;
                } else if let Some(else_stmt) = else_branch {
                    self.evaluate_stmt(scope, else_stmt)?;
                }
            }
            Stmt::WhileLoop { condition, body } => {
                let while_scope = self.env.new_scope(Some(scope));
                while self.evaluate_expr(while_scope, condition)?.is_truthy() {
                    self.evaluate_stmt(while_scope, body)?;
                }
            }
            Stmt::Fun { name, params, body } => {
                let identifier = name.lexeme_str();
                let fun: LoxValue = LoxFunction {
                    name: Some(identifier.clone()),
                    params: params.clone(),
                    body: FunctionBody::Block(body.clone()),
                    closure: Some(self.env.new_scope(Some(scope))),
                    is_method: false,
                }
                .into();
                self.env.declare(Some(scope), identifier, fun);
            }
            Stmt::Return(expr) => {
                let last = self.stack.len() - 1;
                self.stack[last] = self.evaluate_expr(scope, expr)?;
            }
            Stmt::Class { name, methods } => {
                let mut class = LoxClass {
                    name: name.lexeme_str(),
                    constructor: None,
                    methods: HashMap::new(),
                };
                self.env.declare(Some(scope), name.lexeme_str(), class.into());
            }
        }
        Ok(())
    }

    fn evaluate_expr(&mut self, scope: ScopeHandle, expr: &Expr) -> LoxResult<LoxValue> {
        match &expr.kind {
            ExprKind::Literal(value) => Ok(LoxValue::from(value.clone())),
            ExprKind::Unary { operator, right } => match operator.kind {
                TokenKind::Bang => {
                    let right_value = self.evaluate_expr(scope, right)?.is_truthy();
                    Ok(LoxValue::Boolean(!right_value))
                }
                _ => Err(LoxError::Runtime(format!(
                    "Unknown unary operator \"{}\"",
                    operator
                ))),
            },
            ExprKind::Binary {
                operator,
                left,
                right,
            } => self.evaluate_binary_expr(scope, operator, left, right),
            ExprKind::Grouping(inner) => self.evaluate_expr(scope, inner),
            ExprKind::Identifier(name) => {
                let scope = match self.locals.get(&expr.id()) {
                    Some(depth) => self.env.ancestor_scope(scope, *depth).unwrap_or_else(|| {
                        panic!("Invalid ancestor scope for \"{}\"", name.lexeme_str())
                    }),
                    None => GLOBAL_SCOPE,
                };
                self.env
                    .get(Some(scope), &name.lexeme_str())
                    .ok_or(LoxError::Runtime(format!(
                        "Undefined variable \"{}\"",
                        name.lexeme_str()
                    )))
            }
            ExprKind::Assignment { name, value } => {
                let val = self.evaluate_expr(scope, value)?;
                let scope = match self.locals.get(&expr.id()) {
                    Some(distance) => {
                        self.env
                            .ancestor_scope(scope, *distance)
                            .unwrap_or_else(|| {
                                panic!("Invalid ancestor scope for \"{}\"", name.lexeme_str())
                            })
                    }
                    None => GLOBAL_SCOPE,
                };
                self.env.assign(Some(scope), name.lexeme_str(), val.clone());
                Ok(val)
            }
            ExprKind::Logical {
                operator,
                left,
                right,
            } => match operator.kind {
                TokenKind::Or => {
                    let mut val = self.evaluate_expr(scope, left)?;
                    if !val.is_truthy() {
                        val = self.evaluate_expr(scope, right)?;
                    }
                    Ok(val)
                }
                TokenKind::And => {
                    let mut val = self.evaluate_expr(scope, left)?;
                    if val.is_truthy() {
                        val = self.evaluate_expr(scope, right)?;
                    }
                    Ok(val)
                }
                _ => Err(LoxError::Runtime(format!(
                    "Expected logical operator, got \"{}\"",
                    operator.lexeme_str()
                ))),
            },
            ExprKind::Call { callee, arguments } => {
                match self.evaluate_expr(scope, callee)? {
                    LoxValue::Function(fun) => {
                        self.call_func(scope, &fun, arguments, None)
                    },
                    LoxValue::Class(class) => {
                        let mut obj: LoxValue = LoxObject {
                            class: class.clone(),
                            vars: HashMap::new(),
                        }.into();
                        if let Some(constructor) = &class.constructor {
                            self.call_func(scope, constructor, arguments, Some(&mut obj))?;
                        }
                        Ok(obj)
                    },
                    _ => {
                        Err(LoxError::Runtime("Cannot call a non-function".into()))
                    }
                }
            },
            ExprKind::Get { left, right } => {
                Ok(LoxValue::Nil)
            }
            ExprKind::Set { object, identifier, value } => {
                Ok(LoxValue::Nil)
            }
        }
    }

    fn evaluate_binary_expr(
        &mut self,
        scope: ScopeHandle,
        operator: &Token,
        left: &Expr,
        right: &Expr,
    ) -> LoxResult<LoxValue> {
        let left_value = self.evaluate_expr(scope, left)?;
        let right_value = self.evaluate_expr(scope, right)?;
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

    fn call_func(&mut self, scope: ScopeHandle, func: &LoxFunction, arguments: &[Expr], _this: Option<&mut LoxValue>) -> LoxResult<LoxValue> {
        if arguments.len() != func.params.len() {
            Err(LoxError::Runtime(format!(
                "Function \"{}\" takes {} argument(s)",
                func.name.clone().unwrap_or("".into()),
                func.params.len(),
            )))
        } else {
            let mut args: Vec<LoxValue> = vec![];
            for arg in arguments.iter() {
                args.push(self.evaluate_expr(scope, arg)?);
            }
            let return_value = match &func.body {
                FunctionBody::Block(statements) => {
                    let closure = func.closure.expect("Function should have a closure");
                    for (i, arg) in args.drain(0..).enumerate() {
                        self.env.declare(
                            Some(closure),
                            func.params[i].lexeme_str(),
                            arg,
                        );
                    }
                    self.stack.push(LoxValue::Nil);
                    for stmt in statements.iter() {
                        self.evaluate_stmt(closure, stmt)?;
                        if matches!(stmt, Stmt::Return(_)) {
                            break;
                        }
                    }
                    self.stack.pop().unwrap()
                }
                FunctionBody::Native(func) => func(args)?,
            };
            Ok(return_value)
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::test_scripts::*;
    use super::*;
    use mock_logger::MockLogger;

    #[test]
    fn print() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(PRINT_TEST)?;
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
        lox.exec(BLOCK_SCOPE_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "foo");
            assert_eq!(entries[1].body, "bar");
        });
        Ok(())
    }

    #[test]
    fn control_flow() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(CONTROL_FLOW_TEST)?;
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
        lox.exec(WHILE_LOOP_TEST)?;
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
        lox.exec(FOR_LOOP_TEST)?;
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
    fn builtins() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(BUILTINS_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 1);
            assert_ne!(entries[0].body, "nil");
        });
        Ok(())
    }

    #[test]
    fn function() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(FUNCTION_TEST)?;
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
        lox.exec(FUNCTION_CLOSURE_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "1");
            assert_eq!(entries[1].body, "2")
        });
        Ok(())
    }

    #[test]
    fn shadowing() -> LoxResult {
        mock_logger::init();
        let mut lox = Lox::new();
        lox.exec(SHADOWING_TEST)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].body, "global");
            assert_eq!(entries[1].body, "global")
        });
        Ok(())
    }

    // #[test]
    // fn class() -> LoxResult {
    //     mock_logger::init();
    //     let mut lox = Lox::new();
    //     lox.exec(CLASS_TEST)?;
    //     MockLogger::entries(|entries| {
    //         assert_eq!(entries.len(), 1);
    //         assert_eq!(entries[0].body, "Hello, world");
    //     });
    //     Ok(())
    // }
}
