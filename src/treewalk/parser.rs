use super::{
    ast::*,
    error::{LoxError, SyntaxError},
    scanner::*,
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    errors: Vec<LoxError>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: &'a Vec<Token>) -> Result<(), LoxError> {
        let mut _parser = Self {
            tokens,
            errors: vec![],
            current: 0,
        };
        Ok(())
    }

    /**
     * Expression Terminals
     */

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut left = self.comparison()?;
        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut left = self.term()?;
        while self.match_tokens(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right))
        }
        Some(left)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut left = self.factor()?;
        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut left = self.unary()?;
        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_tokens(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Some(Expr::unary(operator, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.match_tokens(&[
            TokenKind::Number,
            TokenKind::String,
            TokenKind::True,
            TokenKind::False,
            TokenKind::Nil,
        ]) {
            Some(Expr::literal(self.advance().clone()))
        } else if self.match_tokens(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected closing ')'");
            Some(Expr::grouping(Box::new(expr)))
        } else {
            None
        }
    }

    /**
     * Utility methods
     */

    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds.iter() {
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn syntax_error(&mut self, error: SyntaxError) {
        self.errors.push(LoxError::SyntaxError(error));
    }

    fn consume(&mut self, kind: TokenKind, err_msg: &str) {
        if self.check(kind) {
            self.advance();
        } else {
            self.syntax_error(SyntaxError::new(
                err_msg.into(),
                self.peek().line,
            ));
        }
    }
}
