use super::scanner::Token;
use std::{
    cell::RefCell,
    fmt::Display
};

thread_local! {
    static EXPR_COUNT: RefCell<usize> = RefCell::new(0);
}

fn get_expr_id() -> usize {
    let mut id = 0;
    EXPR_COUNT.with(|cell| {
        id = cell.take();
        cell.replace(id + 1);
    });
    id
}

#[derive(PartialEq, Clone)]
pub enum ExprKind {
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
    Get {
        left: Box<Expr>,
        right: Token,
    },
    Set {
        object: Box<Expr>,
        identifier: Token,
        value: Box<Expr>,
    },
    This(Token),
}

#[derive(PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    _id: usize,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self {
            kind,
            _id: get_expr_id(),
        }
    }

    pub fn id(&self) -> usize {
        self._id
    }
}

impl From<ExprKind> for Expr {
    fn from(value: ExprKind) -> Self {
        Expr::new(value)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ExprKind::Literal(value) => write!(f, "{}", value.lexeme_str()),
            ExprKind::Unary { operator, right } => {
                write!(
                    f,
                    "({} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    right,
                )
            }
            ExprKind::Binary {
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
            ExprKind::Grouping(inner) => {
                write!(f, "{}", inner)
            }
            ExprKind::Identifier(name) => {
                write!(f, "{}", name.lexeme_str())
            }
            ExprKind::Assignment { name, value } => {
                write!(f, "(= {} {})", name, value)
            }
            ExprKind::Logical {
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
            ExprKind::Call { callee, arguments } => {
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
            ExprKind::Get { left, right } => {
                write!(
                    f,
                    "(property {} {})",
                    left.to_string(),
                    right.lexeme_str()
                )
            }
            ExprKind::Set { object, identifier, value } => {
                write!(
                    f,
                    "(set (property {} {}) {})",
                    object.to_string(),
                    identifier.lexeme_str(),
                    value.to_string()
                )
            }
            ExprKind::This(..) => {
                write!(f, "(this)")
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
    Class {
        name: Token,
        methods: Vec<Stmt>,
    }
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
