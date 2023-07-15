use super::{
    ast::*,
    environment::*,
    error::*,
    scanner::{Literal, Token},
};

pub type NativeFunction = fn(&mut Environment, Vec<LoxValue>) -> LoxResult<LoxValue>;

#[derive(PartialEq, Clone)]
pub enum FunctionBody {
    Block(Vec<Stmt>),
    Native(NativeFunction),
}

#[derive(PartialEq, Clone)]
pub enum LoxValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function {
        name: Option<String>,
        params: Vec<Token>,
        body: FunctionBody,
    }
}

impl LoxValue {
    pub fn from_fn(name: Option<String>, params: Vec<Token>, body: FunctionBody) -> Self {
        Self::Function { name, params, body }
    }

    pub fn type_str(&self) -> String {
        match self {
            Self::Nil => "nil".into(),
            Self::Boolean(_) => "Boolean".into(),
            Self::Number(_) => "Number".into(),
            Self::String(_) => "String".into(),
            Self::Function { name: _, params: _, body: _ } => "Function".into(),
        }
    }

    pub fn _is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn _is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_fun(&self) -> bool {
        matches!(
            self,
            Self::Function { name: _, params: _, body: _ }
        )
    }

    pub fn _set_nil(&mut self) {
        *self = Self::Nil;
    }

    pub fn _get_boolean(&self) -> LoxResult<bool> {
        if let Self::Boolean(value) = self {
            Ok(*value)
        } else {
            Err(LoxError::RuntimeError(format!(
                "Expected Boolean, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn _set_boolean(&mut self, value: bool) {
        *self = Self::Boolean(value);
    }

    pub fn get_number(&self) -> LoxResult<f64> {
        if let Self::Number(value) = self {
            Ok(*value)
        } else {
            Err(LoxError::RuntimeError(format!(
                "Expected Number, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn _set_number(&mut self, value: f64) {
        *self = Self::Number(value);
    }

    pub fn _get_string(&self) -> LoxResult<String> {
        if let Self::String(value) = self {
            println!("get_string(): {}", value);
            Ok(value.clone())
        } else {
            Err(LoxError::RuntimeError(format!(
                "Expected String, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn _set_string(&mut self, value: String) {
        *self = Self::String(value);
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(value) => *value,
            _ => true,
        }
    }
}

impl From<bool> for LoxValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for LoxValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for LoxValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl ToString for LoxValue {
    fn to_string(&self) -> String {
        match self {
            Self::Nil => "nil".into(),
            Self::Boolean(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::String(value) => value.clone(),
            Self::Function { name, params: _, body: _ } => {
                format!("<function {}>", name.as_ref().unwrap_or(&"".into()))
            }

        }
    }
}

impl From<Token> for LoxValue {
    fn from(token: Token) -> Self {
        match token.literal {
            Some(literal) => match literal {
                Literal::False => Self::Boolean(false),
                Literal::True => Self::Boolean(true),
                Literal::Number(num) => Self::Number(num),
                Literal::String(s) => Self::String(s),
            },
            None => Self::Nil,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(LoxValue::Nil.to_string(), "nil");
        assert_eq!(LoxValue::Boolean(true).to_string(), "true");
        assert_eq!(LoxValue::Number(3.14).to_string(), "3.14");
        assert_eq!(LoxValue::String("foo".to_string()).to_string(), "foo");
    }
}
