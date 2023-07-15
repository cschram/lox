use super::scanner::Token;
use std::fmt::Display;

#[derive(PartialEq, Clone)]
pub enum Expr {
    Literal(Token),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Identifier(Token),
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(value) => write!(f, "{}", value.lexeme_str()),
            Expr::Unary { operator, right } => {
                write!(
                    f,
                    "({} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    right,
                )
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    left,
                    right
                )
            }
            Expr::Grouping(inner) => {
                write!(f, "{}", inner)
            }
            Expr::Identifier(name) => {
                write!(f, "{}", name.lexeme_str())
            }
            Expr::Assignment { name, value } => {
                write!(f, "(= {} {})", name, value)
            }
            Expr::Logical {
                operator,
                left,
                right,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    left,
                    right
                )
            }
            Expr::Call { callee, arguments } => {
                write!(
                    f,
                    "(call {} {})",
                    callee,
                    arguments
                        .iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
        }
    }
}

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
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            }
        }
    }
}
