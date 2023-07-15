use super::scanner::{Token, TokenKind};
use std::fmt::Display;

#[derive(Debug)]
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

impl Expr {
    pub fn literal(value: Token) -> Self {
        Expr::assert_token_kind(
            &value,
            &[
                TokenKind::Number,
                TokenKind::String,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Nil,
            ],
            "Expected a literal token",
        );
        Self::Literal(value)
    }

    pub fn unary(operator: Token, right: Box<Expr>) -> Self {
        Expr::assert_token_kind(
            &operator,
            &[TokenKind::Bang, TokenKind::Minus],
            "Expected unary operator",
        );
        Self::Unary { operator, right }
    }

    pub fn binary(operator: Token, left: Box<Expr>, right: Box<Expr>) -> Self {
        Expr::assert_token_kind(
            &operator,
            &[
                TokenKind::BangEqual,
                TokenKind::EqualEqual,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
                TokenKind::Minus,
                TokenKind::Plus,
                TokenKind::Slash,
                TokenKind::Star,
            ],
            "Expected binary operator",
        );
        Self::Binary {
            operator,
            left,
            right,
        }
    }

    pub fn grouping(inner: Box<Expr>) -> Self {
        Self::Grouping(inner)
    }

    pub fn identifier(name: Token) -> Self {
        Self::Identifier(name)
    }

    pub fn assignment(name: Token, value: Box<Expr>) -> Self {
        Expr::assert_token_kind(&name, &[TokenKind::Identifier], "Expected identifier");
        Self::Assignment { name, value }
    }

    pub fn logical(operator: Token, left: Box<Expr>, right: Box<Expr>) -> Self {
        Expr::assert_token_kind(
            &operator,
            &[TokenKind::Or, TokenKind::And],
            "Expected logical operator",
        );
        Self::Logical {
            operator,
            left,
            right,
        }
    }

    pub fn call(callee: Box<Expr>, arguments: Vec<Expr>) -> Self {
        Self::Call { callee, arguments }
    }

    fn assert_token_kind(token: &Token, kinds: &[TokenKind], err_msg: &str) {
        for kind in kinds.iter() {
            if token.kind == *kind {
                return;
            }
        }
        panic!("{}", err_msg);
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(value) => write!(f, "{}", value.lexeme.as_ref().unwrap()),
            Expr::Unary { operator, right } => {
                write!(
                    f,
                    "({} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    right.to_string(),
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
                    left.to_string(),
                    right.to_string()
                )
            }
            Expr::Grouping(inner) => {
                write!(f, "{}", inner.to_string(),)
            }
            Expr::Identifier(name) => {
                write!(f, "{}", name.lexeme.as_ref().unwrap())
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
                    left.to_string(),
                    right.to_string()
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
}

impl Stmt {
    pub fn expr(expr: Box<Expr>) -> Self {
        Self::Expr(expr)
    }

    pub fn print(expr: Box<Expr>) -> Self {
        Self::Print(expr)
    }

    pub fn var(name: Token, initializer: Option<Box<Expr>>) -> Self {
        Self::Var { name, initializer }
    }

    pub fn block(statements: Vec<Stmt>) -> Self {
        Self::Block(statements)
    }

    pub fn if_else(condition: Box<Expr>, body: Box<Stmt>, else_branch: Option<Box<Stmt>>) -> Self {
        Self::IfElse {
            condition,
            body,
            else_branch,
        }
    }

    pub fn while_loop(condition: Box<Expr>, body: Box<Stmt>) -> Self {
        Self::WhileLoop { condition, body }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expr(expr) => write!(f, "({})", expr.to_string()),
            Self::Print(expr) => write!(f, "(print {})", expr.to_string()),
            Self::Var { name, initializer } => match initializer {
                Some(expr) => write!(
                    f,
                    "(var {} {})",
                    name.lexeme.as_ref().unwrap().to_string(),
                    expr.to_string()
                ),
                None => write!(f, "(var {})", name.lexeme.as_ref().unwrap().to_string()),
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
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::scanner::*;
    use super::*;

    #[test]
    fn expr_to_string() {
        let expr = Expr::binary(
            Token::new(TokenKind::Plus, Some("+".into()), None, 0),
            Box::new(Expr::unary(
                Token::new(TokenKind::Minus, Some("-".into()), None, 0),
                Box::new(Expr::literal(Token::new(
                    TokenKind::Number,
                    Some("6.5".into()),
                    Some(Literal::Number(6.5)),
                    0,
                ))),
            )),
            Box::new(Expr::identifier(Token::new(
                TokenKind::Identifier,
                Some("foo".into()),
                None,
                0,
            ))),
        );
        assert_eq!(expr.to_string(), "(+ (- 6.5) foo)".to_owned());
    }

    #[test]
    fn stmt_to_string() {
        let expr_stmt = Stmt::expr(Box::new(Expr::binary(
            Token::new(TokenKind::Plus, Some("+".into()), None, 0),
            Box::new(Expr::literal(Token::new(
                TokenKind::String,
                Some(r#""pi = ""#.into()),
                Some(Literal::String(r#""pi = ""#.into())),
                0,
            ))),
            Box::new(Expr::literal(Token::new(
                TokenKind::Number,
                Some("3.14".into()),
                Some(Literal::Number(3.14)),
                0,
            ))),
        )));
        assert_eq!(expr_stmt.to_string(), r#"((+ "pi = " 3.14))"#);
        let print_stmt = Stmt::print(Box::new(Expr::binary(
            Token::new(TokenKind::Plus, Some("+".into()), None, 0),
            Box::new(Expr::literal(Token::new(
                TokenKind::String,
                Some(r#""pi = ""#.into()),
                Some(Literal::String(r#""pi = ""#.into())),
                0,
            ))),
            Box::new(Expr::literal(Token::new(
                TokenKind::Number,
                Some("3.14".into()),
                Some(Literal::Number(3.14)),
                0,
            ))),
        )));
        assert_eq!(print_stmt.to_string(), r#"(print (+ "pi = " 3.14))"#);
        let var_stmt = Stmt::var(
            Token::new(TokenKind::Identifier, Some("foo".into()), None, 0),
            Some(Box::new(Expr::Literal(Token::new(
                TokenKind::String,
                Some(r#""bar""#.into()),
                Some(Literal::String(r#""bar""#.into())),
                0,
            )))),
        );
        assert_eq!(var_stmt.to_string(), r#"(var foo "bar")"#);
    }
}
