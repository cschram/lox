use super::{
    environment::ScopeHandle,
    error::*,
    expr::Expr,
    scanner::Token,
    state::LoxState,
    value::{LoxValue, LoxFunction, LoxClass},
};
use std::{fmt, collections::HashMap};
use log::info;

#[derive(PartialEq, Clone)]
pub enum Stmt {
    Expr(Box<Expr>),
    Print(Box<Expr>),
    Var {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    Block(Vec<Stmt>),
    IfElse {
        condition: Box<Expr>,
        body: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    WhileLoop {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Fun {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return(Box<Expr>),
    Class {
        name: Token,
        methods: Vec<Stmt>,
    }
}

impl Stmt {
    pub fn eval(&self, state: &mut LoxState, scope: ScopeHandle) -> LoxResult {
        match self {
            Stmt::Expr(expr) => {
                expr.eval(state, scope)?;
            }
            Stmt::Print(expr) => {
                let value = expr.eval(state, scope)?;
                info!("{}", value.to_string());
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => expr.eval(state, scope)?,
                    None => LoxValue::Nil,
                };
                state.env.declare(Some(scope), name.lexeme_str(), value);
            }
            Stmt::Block(statements) => {
                let block_scope = state.env.new_scope(Some(scope));
                for stmt in statements.iter() {
                    stmt.eval(state, block_scope)?;
                }
            }
            Stmt::IfElse {
                condition,
                body,
                else_branch,
            } => {
                let cond = condition.eval(state, scope)?;
                if cond.is_truthy() {
                    body.eval(state, scope)?;
                } else if let Some(else_stmt) = else_branch {
                    else_stmt.eval(state, scope)?;
                }
            }
            Stmt::WhileLoop { condition, body } => {
                let while_scope = state.env.new_scope(Some(scope));
                while condition.eval(state, while_scope)?.is_truthy() {
                    body.eval(state, while_scope)?;
                }
            }
            Stmt::Fun { name, .. } => {
                let fun = LoxFunction::from_stmt(self, state.env.new_scope(Some(scope)))?;
                state.env.declare(Some(scope), name.lexeme_str(), fun.into());
            }
            Stmt::Return(expr) => {
                let last = state.stack.len() - 1;
                state.stack[last] = expr.eval(state, scope)?;
            }
            Stmt::Class { name, methods: method_defs } => {
                let mut methods = HashMap::<String, LoxFunction>::new();
                for def in method_defs.iter() {
                    let fun = LoxFunction::from_stmt(def, scope)?;
                    methods.insert(fun.name.clone().unwrap(), fun);
                }
                state.env.declare(Some(scope), name.lexeme_str(), LoxClass {
                    name: name.lexeme_str(),
                    methods,
                }.into());
            }
        }
        Ok(())
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expr(expr) => write!(f, "({})", expr),
            Self::Print(expr) => write!(f, "(print {})", expr),
            Self::Var { name, initializer } => match initializer {
                Some(expr) => write!(f, "(var {} {})", name.lexeme_str(), expr),
                None => write!(f, "(var {})", name.lexeme_str()),
            },
            Self::Block(statements) => {
                write!(f, "(block ")?;
                for stmt in statements.iter() {
                    write!(f, "{}", stmt)?;
                }
                write!(f, ")")
            }
            Self::IfElse {
                condition,
                body,
                else_branch,
            } => match else_branch {
                Some(else_stmt) => {
                    write!(f, "(if {} {} else {}", condition, body, else_stmt)
                }
                None => {
                    write!(f, "(if {} {})", condition, body)
                }
            },
            Self::WhileLoop { condition, body } => {
                write!(f, "(while {} {}", condition, body)
            }
            Self::Fun { name, params, body } => {
                write!(
                    f,
                    "(fun {} ({}) ({}))",
                    name.lexeme_str(),
                    params
                        .iter()
                        .map(|param| param.lexeme_str())
                        .collect::<Vec<String>>()
                        .join(" "),
                    body.iter()
                        .map(|stmt| stmt.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Self::Return(value) => {
                write!(f, "(return {})", value)
            },
            Self::Class { name, methods } => {
                write!(
                    f,
                    "(class {} ({}))",
                    name.lexeme_str(),
                    methods.iter()
                        .map(|stmt| stmt.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )

            }
        }
    }
}
