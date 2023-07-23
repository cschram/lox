use std::{fmt::Display, mem::take};

use super::error::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::String(s) => write!(f, "{}", s),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Option<String>,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        lexeme: Option<String>,
        literal: Option<Literal>,
        line: u32,
    ) -> Self {
        Self {
            kind,
            lexeme,
            literal,
            line,
        }
    }

    pub fn lexeme_str(&self) -> String {
        match &self.lexeme {
            Some(lexeme) => lexeme.clone(),
            None => "".into(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?}, {:?})", self.kind, self.lexeme, self.literal)
    }
}

pub struct ScanResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<SyntaxError>,
}

// Lexical Scanner
// Produces tokens
pub struct Scanner {
    // Source code, as a vector of characters
    source: Vec<char>,
    // Scanned tokens
    tokens: Vec<Token>,
    // Syntax errors
    errors: Vec<SyntaxError>,
    // Current line being scanned
    line: usize,
    // Starting offset of current lexeme being scanned
    start: usize,
    // Current offset of the lexeme being scanned
    current: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],
            errors: vec![],
            line: 0,
            start: 0,
            current: 0,
        }
    }

    // Do a full scan of the source.
    pub fn scan(&mut self) -> ScanResult {
        while !self.id_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(TokenKind::Eof, None, None, self.line as u32 + 1));
        ScanResult {
            tokens: take(&mut self.tokens),
            errors: take(&mut self.errors),
        }
    }

    // Scan a single token.
    fn scan_token(&mut self) {
        match self.advance() {
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '(' => self.add_token(TokenKind::LeftParen, None),
            ')' => self.add_token(TokenKind::RightParen, None),
            '{' => self.add_token(TokenKind::LeftBrace, None),
            '}' => self.add_token(TokenKind::RightBrace, None),
            ',' => self.add_token(TokenKind::Comma, None),
            '.' => self.add_token(TokenKind::Dot, None),
            '-' => self.add_token(TokenKind::Minus, None),
            '+' => self.add_token(TokenKind::Plus, None),
            ';' => self.add_token(TokenKind::Semicolon, None),
            '*' => self.add_token(TokenKind::Star, None),
            '!' => {
                if *self.peek() == '=' {
                    self.add_token(TokenKind::BangEqual, None);
                    self.advance();
                } else {
                    self.add_token(TokenKind::Bang, None);
                }
            }
            '=' => {
                if *self.peek() == '=' {
                    self.add_token(TokenKind::EqualEqual, None);
                    self.advance();
                } else {
                    self.add_token(TokenKind::Equal, None);
                }
            }
            '<' => {
                if *self.peek() == '=' {
                    self.add_token(TokenKind::LessEqual, None);
                    self.advance();
                } else {
                    self.add_token(TokenKind::Less, None);
                }
            }
            '>' => {
                if *self.peek() == '=' {
                    self.add_token(TokenKind::GreaterEqual, None);
                    self.advance();
                } else {
                    self.add_token(TokenKind::Greater, None);
                }
            }
            '/' => {
                if *self.peek() == '/' {
                    self.scan_comment();
                } else {
                    self.add_token(TokenKind::Slash, None);
                }
            }
            '"' => self.scan_string(),
            '0'..='9' => self.scan_number(),
            _ => {
                if self.previous().is_alphabetic() {
                    self.scan_identifier();
                } else {
                    self.add_syntax_error(format!("Unknown character \"{}\"", self.previous()));
                }
            }
        }
    }

    // Ignore a comment line and advance to the next line.
    fn scan_comment(&mut self) {
        while *self.peek() != '\n' && !self.id_at_end() {
            self.advance();
        }
    }

    // Scan a string token.
    fn scan_string(&mut self) {
        let mut line = self.line;
        while *self.peek() != '"' && !self.id_at_end() {
            if *self.peek() == '\n' {
                line += 1;
            }
            self.advance();
        }
        if self.id_at_end() {
            self.add_syntax_error("Unterminated string".to_owned());
        } else {
            self.advance();
            let lexeme = self.get_lexeme();
            let literal = lexeme[1..lexeme.len() - 1].to_string();
            self.tokens.push(Token::new(
                TokenKind::String,
                Some(lexeme),
                Some(Literal::String(literal)),
                self.line as u32,
            ));
            self.line = line;
        }
    }

    // Scan a number token.
    fn scan_number(&mut self) {
        while !self.id_at_end() && self.is_digit() {
            self.advance();
        }
        let s = self.get_lexeme();
        let num = s.parse::<f64>().expect("Invalid number");
        self.tokens.push(Token::new(
            TokenKind::Number,
            Some(s),
            Some(Literal::Number(num)),
            self.line as u32,
        ));
    }

    // Scan an identifier
    fn scan_identifier(&mut self) {
        while !self.id_at_end() && (self.peek().is_alphanumeric() || *self.peek() == '_') {
            self.advance();
        }
        let lexeme = self.get_lexeme();
        let kind = match lexeme.as_str() {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };
        let literal = match kind {
            TokenKind::True => Some(Literal::True),
            TokenKind::False => Some(Literal::False),
            _ => None,
        };
        self.add_token(kind, literal);
    }

    // Add a token
    fn add_token(&mut self, kind: TokenKind, literal: Option<Literal>) {
        self.tokens.push(Token::new(
            kind,
            Some(self.get_lexeme()),
            literal,
            self.line as u32,
        ));
    }

    // Grab the current character.
    fn peek(&self) -> &char {
        &self.source[self.current]
    }

    // Grab the last character.
    fn previous(&self) -> &char {
        &self.source[self.current - 1]
    }

    // Grab the next character.
    fn peek_next(&self) -> Option<&char> {
        if (self.current + 1) < self.source.len() {
            Some(&self.source[self.current + 1])
        } else {
            None
        }
    }

    // Check if the current charater is a digit.
    // If the current character is a dot (".") it will check if the next
    // character is a digit to verify if the dot is meant as a decimal.
    fn is_digit(&self) -> bool {
        if self.peek().is_ascii_digit() {
            true
        } else if *self.peek() == '.' {
            if let Some(next) = self.peek_next() {
                next.is_ascii_digit()
            } else {
                false
            }
        } else {
            false
        }
    }

    // Consumes the current character, returning it and incrementing
    // the character pointer.
    fn advance(&mut self) -> &char {
        let c = &self.source[self.current];
        self.current += 1;
        c
    }

    // Add a syntax error.
    fn add_syntax_error(&mut self, message: String) {
        self.errors
            .push(SyntaxError::new(message, self.line as u32));
    }

    // Generate the current token lexeme.
    fn get_lexeme(&self) -> String {
        self.source[self.start..self.current].iter().collect()
    }

    // Check if we've reached the end of the source.
    fn id_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

pub fn scan(source: &str) -> ScanResult {
    let mut scanner = Scanner::new(source);
    scanner.scan()
}

#[cfg(test)]
mod test {
    use super::super::super::test_scripts::*;
    use super::*;

    #[test]
    fn expressions() {
        let ScanResult { tokens, errors } = scan(EXPRESSION_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 18);
    }

    #[test]
    fn variables() {
        let ScanResult { tokens, errors } = scan(VARIABLE_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 16);
    }

    #[test]
    fn print() {
        let ScanResult { tokens, errors } = scan(PRINT_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 15);
    }

    #[test]
    fn block_scope() {
        let ScanResult { tokens, errors } = scan(BLOCK_SCOPE_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 19);
    }

    #[test]
    fn control_flow() {
        let ScanResult { tokens, errors } = scan(CONTROL_FLOW_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 37);
    }

    #[test]
    fn function() {
        let ScanResult { tokens, errors } = scan(FUNCTION_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 42);
    }

    #[test]
    fn class() {
        let ScanResult { tokens, errors } = scan(CLASS_TEST);
        for err in errors.iter() {
            println!("{}", err);
        }
        assert_eq!(errors.len(), 0);
        assert_eq!(tokens.len(), 64);
    }
}
