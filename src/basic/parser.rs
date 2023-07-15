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
        if self.match_tokens(&[TokenKind::Fun]) {
            self.function()
        } else if self.match_tokens(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn function(&mut self) -> LoxResult<Stmt> {
        let name = self
            .consume(TokenKind::Identifier, "Expected identifier")?
            .clone();
        self.consume(TokenKind::LeftParen, "Expected opening parenthesis")?;
        let params: Vec<Token> = self.fun_parameters()?;
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        self.consume(TokenKind::LeftBrace, "Expected opening brace")?;
        let mut body: Vec<Stmt> = vec![];
        while !self.match_tokens(&[TokenKind::RightBrace]) && !self.is_at_end() {
            body.push(self.declaration()?);
        }
        Ok(Stmt::Fun { name, params, body })
    }

    fn fun_parameters(&mut self) -> LoxResult<Vec<Token>> {
        if self.match_tokens(&[TokenKind::Identifier]) {
            let mut params = vec![self.previous().clone()];
            while self.match_tokens(&[TokenKind::Identifier]) {
                self.consume(TokenKind::Comma, "Expected comma")?;
                params.push(self.previous().clone());
            }
            Ok(params)
        } else {
            Ok(vec![])
        }
    }

    fn var_declaration(&mut self) -> LoxResult<Stmt> {
        let identifier = self
            .consume(TokenKind::Identifier, "Expected identifier")?
            .clone();
        let var = if self.match_tokens(&[TokenKind::Equal]) {
            let expr = self.expression()?;
            Stmt::Var {
                name: identifier,
                initializer: Some(Box::new(expr)),
            }
        } else {
            Stmt::Var {
                name: identifier,
                initializer: None,
            }
        };
        self.consume(TokenKind::Semicolon, "Expected a semicolon")?;
        Ok(var)
    }

    fn statement(&mut self) -> LoxResult<Stmt> {
        if self.match_tokens(&[TokenKind::For]) {
            self.for_statement()
        } else if self.match_tokens(&[TokenKind::If]) {
            self.if_statement()
        } else if self.match_tokens(&[TokenKind::Print]) {
            self.print_statement()
        } else if self.match_tokens(&[TokenKind::Return]) {
            self.return_statement()
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
            self.var_declaration()?
        } else {
            self.expression_statement()?
        };
        let condition = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected semicolon")?;
        let iterator = self.expression()?;
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        let body = self.statement()?;
        Ok(Stmt::Block(vec![
            initializer,
            Stmt::WhileLoop {
                condition: Box::new(condition),
                body: Box::new(Stmt::Block(vec![body, Stmt::Expr(Box::new(iterator))])),
            },
        ]))
    }

    fn if_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected opening parenthesis")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        let body = Box::new(self.statement()?);
        if self.match_tokens(&[TokenKind::Else]) {
            let else_branch = Box::new(self.statement()?);
            Ok(Stmt::IfElse {
                condition,
                body,
                else_branch: Some(else_branch),
            })
        } else {
            Ok(Stmt::IfElse {
                condition,
                body,
                else_branch: None,
            })
        }
    }

    fn print_statement(&mut self) -> LoxResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected a semicolon")?;
        Ok(Stmt::Print(Box::new(expr)))
    }

    fn return_statement(&mut self) -> LoxResult<Stmt> {
        let value = if self.check(TokenKind::Semicolon) {
            Expr::Literal(Token::new(
                TokenKind::Nil,
                Some("nil".to_string()),
                None,
                self.previous().line,
            ))
        } else {
            self.expression()?
        };
        self.consume(TokenKind::Semicolon, "Expected a semicolon")?;
        Ok(Stmt::Return(Box::new(value)))
    }

    fn while_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected opening parenthesis")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::WhileLoop { condition, body })
    }

    fn block(&mut self) -> LoxResult<Stmt> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenKind::RightBrace, "Expected closing brace")?;
        Ok(Stmt::Block(statements))
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
                left = Expr::Assignment {
                    name,
                    value: Box::new(right),
                };
            } else {
                return Err(LoxError::Runtime("Invalid assignment target".into()));
            }
        }
        Ok(left)
    }

    fn logic_or(&mut self) -> LoxResult<Expr> {
        let mut left = self.logic_and()?;
        while self.match_tokens(&[TokenKind::Or]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            left = Expr::Logical {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn logic_and(&mut self) -> LoxResult<Expr> {
        let mut left = self.equality()?;
        while self.match_tokens(&[TokenKind::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            left = Expr::Logical {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn equality(&mut self) -> LoxResult<Expr> {
        let mut left = self.comparison()?;
        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
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
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn term(&mut self) -> LoxResult<Expr> {
        let mut left = self.factor()?;
        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn factor(&mut self) -> LoxResult<Expr> {
        let mut left = self.unary()?;
        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> LoxResult<Expr> {
        if self.match_tokens(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
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
                        return Err(LoxError::Runtime(
                            "Exceeded maximum number of arguments".into(),
                        ));
                    }
                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.consume(TokenKind::RightParen, "Expected closing parenthesis")?;
            }
            left = Expr::Call {
                callee: Box::new(left),
                arguments,
            };
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
            Ok(Expr::Literal(self.previous().clone()))
        } else if self.match_tokens(&[TokenKind::Identifier]) {
            Ok(Expr::Identifier(self.previous().clone()))
        } else if self.match_tokens(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected closing ')'")?;
            Ok(Expr::Grouping(Box::new(expr)))
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
        LoxError::Syntax(SyntaxError::new(message.into(), line))
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
        );
        let ParseResult { statements, errors } = Parser::parse(tokens);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 3);
    }
}
