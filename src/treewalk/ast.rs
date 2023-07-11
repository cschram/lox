use super::scanner::Token;
use log::info;
use std::fmt::Display;

#[derive(Debug)]
pub enum Expr {
    Literal {
        value: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping {
        inner: Box<Expr>,
    },
}

impl Expr {
    pub fn literal(value: Token) -> Self {
        Self::Literal { value }
    }
    

    pub fn unary(operator: Token, right: Box<Expr>) -> Self {
        Self::Unary { operator, right }
    }

    pub fn binary(operator: Token, left: Box<Expr>, right: Box<Expr>) -> Self {
        Self::Binary {
            operator,
            left,
            right,
        }
    }

    pub fn grouping(inner: Box<Expr>) -> Self {
        Self::Grouping { inner }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal { value } => {
                write!(f, "{}", value.lexeme.clone().unwrap_or("".to_owned()),)
            }
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
            Expr::Grouping { inner } => {
                write!(f, "({})", inner.to_string(),)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::scanner::*;
    use super::*;

    #[test]
    fn test_expr_to_string() {
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
            Box::new(Expr::literal(Token::new(
                TokenKind::Number,
                Some("10.8".into()),
                Some(Literal::Number(10.8)),
                0,
            ))),
        );
        assert_eq!(expr.to_string(), "(+ (- 6.5) 10.8)".to_owned());
    }
}
