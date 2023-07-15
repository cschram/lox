use super::{ast::*, error::*, scanner::*};

const MAX_ARGUMENTS: usize = 255;

pub struct ParseResult {
    pub statements: Vec<Stmt>,
    pub errors: Vec<LoxError>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> ParseResult {
        let mut parser = Self { tokens, current: 0 };
        let mut statements: Vec<Stmt> = vec![];
        let mut errors: Vec<LoxError> = vec![];
        while !parser.is_at_end() {
            if !parser.match_tokens(&[TokenKind::Eof]) {
                match parser.declaration() {
                    Ok(stmt) => {
                        statements.push(stmt);
                    }
                    Err(err) => {
                        errors.push(err);
                        parser.synchronize();
                    }
                }
            }
        }
        ParseResult { statements, errors }
    }

    /**
     * Statements
     */
    fn declaration(&mut self) -> LoxResult<Stmt> {
        if self.match_tokens(&[TokenKind::Var]) {
            self.var_statement()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> LoxResult<Stmt> {
        if self.match_tokens(&[TokenKind::For]) {
            self.for_statement()
        } else if self.match_tokens(&[TokenKind::If]) {
            self.if_statement()
        } else if self.match_tokens(&[TokenKind::Print]) {
            self.print_statement()
        } else if self.match_tokens(&[TokenKind::While]) {
            self.while_statement()
        } else if self.match_tokens(&[TokenKind::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn expression_statement(&mut self) -> LoxResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected a semicolon")?;
        Ok(Stmt::Expr(Box::new(expr)))
    }

    fn for_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected opening parenthesis")?;
        let initializer = if self.match_tokens(&[TokenKind::Var]) {
            self.var_statement()?
        } else {
            self.expression_statement()?
        };
        let condition = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected semicolon")?;
        let iterator = self.expression()?;
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        let body = self.statement()?;
        Ok(Stmt::block(vec![
            initializer,
            Stmt::while_loop(
                Box::new(condition),
                Box::new(Stmt::block(vec![body, Stmt::expr(Box::new(iterator))])),
            ),
        ]))
        // Ok(Stmt::for_loop(initializer, condition, iterator, body))
    }

    fn if_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected opening parenthesis")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        let body = Box::new(self.statement()?);
        if self.match_tokens(&[TokenKind::Else]) {
            let else_branch = Box::new(self.statement()?);
            Ok(Stmt::if_else(condition, body, Some(else_branch)))
        } else {
            Ok(Stmt::if_else(condition, body, None))
        }
    }

    fn print_statement(&mut self) -> LoxResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected a semicolon")?;
        Ok(Stmt::Print(Box::new(expr)))
    }

    fn while_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected opening parenthesis")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::while_loop(condition, body))
    }

    fn var_statement(&mut self) -> LoxResult<Stmt> {
        let identifier = self
            .consume(TokenKind::Identifier, "Expected identifier")?
            .clone();
        let var = if self.match_tokens(&[TokenKind::Equal]) {
            let expr = self.expression()?;
            Stmt::var(identifier, Some(Box::new(expr)))
        } else {
            Stmt::var(identifier, None)
        };
        self.consume(TokenKind::Semicolon, "Expected a semicolon")?;
        Ok(var)
    }

    fn block(&mut self) -> LoxResult<Stmt> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenKind::RightBrace, "Expected closing brace")?;
        Ok(Stmt::block(statements))
    }

    /**
     * Expressions
     */

    fn expression(&mut self) -> LoxResult<Expr> {
        self.assignemnt()
    }

    fn assignemnt(&mut self) -> LoxResult<Expr> {
        let mut left = self.logic_or()?;
        if self.match_tokens(&[TokenKind::Equal]) {
            if let Expr::Identifier(name) = left {
                let right = self.assignemnt()?;
                left = Expr::assignment(name, Box::new(right));
            } else {
                return Err(LoxError::RuntimeError("Invalid assignment target".into()));
            }
        }
        Ok(left)
    }

    fn logic_or(&mut self) -> LoxResult<Expr> {
        let mut left = self.logic_and()?;
        while self.match_tokens(&[TokenKind::Or]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            left = Expr::logical(operator, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn logic_and(&mut self) -> LoxResult<Expr> {
        let mut left = self.equality()?;
        while self.match_tokens(&[TokenKind::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            left = Expr::logical(operator, Box::new(left), Box::new(right));
        }
        Ok(left)
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
            self.call()
        }
    }

    fn call(&mut self) -> LoxResult<Expr> {
        let mut left = self.primary()?;
        while self.match_tokens(&[TokenKind::LeftParen]) {
            let mut arguments: Vec<Expr> = vec![];
            if !self.match_tokens(&[TokenKind::RightParen]) {
                loop {
                    arguments.push(self.expression()?);
                    if arguments.len() > MAX_ARGUMENTS {
                        return Err(LoxError::RuntimeError(
                            "Exceeded maximum number of arguments".into(),
                        ));
                    }
                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
            }
            left = Expr::call(Box::new(left), arguments);
        }
        Ok(left)
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
        } else if self.match_tokens(&[TokenKind::Identifier]) {
            Ok(Expr::identifier(self.previous().clone()))
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

    fn consume(&mut self, kind: TokenKind, err_msg: &str) -> LoxResult<&Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.syntax_error(err_msg, self.peek().line))
        }
    }

    fn synchronize(&mut self) {
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

#[cfg(test)]
mod test {
    use super::super::{ScanResult, Scanner};
    use super::*;

    #[test]
    fn print_var() {
        let ScanResult { tokens, errors: _ } = Scanner::scan(
            r#"
            var pi = 3.14;
            print pi;
        "#,
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn block_scope() {
        let ScanResult { tokens, errors: _ } = Scanner::scan(
            r#"
            var foo = "foo";
            print foo;
            {
                var foo = "bar";
                print foo;
            }
        "#,
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 3);
    }

    #[test]
    fn control_flow() {
        let ScanResult { tokens, errors: _ } = Scanner::scan(
            r#"
            if (true and ("true" or 42)) {
                print "true";
            } else {
                print "false";
            }
        "#,
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn while_loop() {
        let ScanResult { tokens, errors: _ } = Scanner::scan(
            r#"
            var i = 4;
            while (i > 0) {
                print i;
                i = i - 1;
            }
        "#,
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn for_loop() {
        let ScanResult { tokens, errors: _ } = Scanner::scan(
            r#"
            for (var i = 0; i < 4; i = i + 1) {
                print i;
            }
        "#,
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn functions() {
        let ScanResult { tokens, errors: _ } = Scanner::scan(
            r#"
            do_thing()("one")(true, false);
        "#,
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);
    }
}
