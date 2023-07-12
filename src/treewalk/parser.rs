use super::{ast::*, error::*, scanner::*};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: &'a Vec<Token>) -> LoxResult<Expr> {
        let mut parser = Self { tokens, current: 0 };
        parser.expression()
    }

    /**
     * Expression Terminals
     */

    fn expression(&mut self) -> LoxResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> LoxResult<Expr> {
        let mut left = self.comparison()?;
        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn comparison(&mut self) -> LoxResult<Expr> {
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
        Ok(left)
    }

    fn term(&mut self) -> LoxResult<Expr> {
        let mut left = self.factor()?;
        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn factor(&mut self) -> LoxResult<Expr> {
        let mut left = self.unary()?;
        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            left = Expr::binary(operator, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> LoxResult<Expr> {
        if self.match_tokens(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::unary(operator, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> LoxResult<Expr> {
        if self.match_tokens(&[
            TokenKind::Number,
            TokenKind::String,
            TokenKind::True,
            TokenKind::False,
            TokenKind::Nil,
        ]) {
            Ok(Expr::literal(self.previous().clone()))
        } else if self.match_tokens(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected closing ')'")?;
            Ok(Expr::grouping(Box::new(expr)))
        } else {
            Err(self.syntax_error("Expected expression", self.peek().line))
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

    fn syntax_error(&self, message: &str, line: u32) -> LoxError {
        LoxError::SyntaxError(SyntaxError::new(message.into(), line))
    }

    fn consume(&mut self, kind: TokenKind, err_msg: &str) -> LoxResult {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax_error(err_msg, self.peek().line))
        }
    }

    fn _synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon
                || matches!(
                    self.peek().kind,
                    TokenKind::Class
                        | TokenKind::Fun
                        | TokenKind::Var
                        | TokenKind::For
                        | TokenKind::If
                        | TokenKind::While
                        | TokenKind::Print
                        | TokenKind::Return
                )
            {
                return;
            }
            self.advance();
        }
    }
}
