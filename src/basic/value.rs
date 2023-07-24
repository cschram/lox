use std::collections::HashMap;

use super::{ast::*, environment::*, error::*, scanner::*};

pub type NativeFunction = fn(Vec<LoxValue>) -> LoxResult<LoxValue>;

#[derive(PartialEq, Clone)]
pub enum FunctionBody {
    Block(Vec<Stmt>),
    Native(NativeFunction),
}

#[derive(PartialEq, Clone)]
pub struct LoxFunction {
    pub name: Option<String>,
    pub params: Vec<Token>,
    pub body: FunctionBody,
    pub closure: Option<ScopeHandle>,
    pub is_method: bool,
}

impl LoxFunction {
    pub fn native(name: &str, params: Vec<&str>, body: NativeFunction) -> Self {
        LoxFunction {
            name: Some(name.into()),
            params: params
                .into_iter()
                .map(|param| Token::new(TokenKind::Identifier, Some(param.into()), None, 0))
                .collect(),
            body: FunctionBody::Native(body),
            closure: None,
            is_method: false,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct LoxClass {
    pub name: String,
    pub constructor: Option<LoxFunction>,
    pub methods: HashMap<String, LoxFunction>,
}

#[derive(PartialEq, Clone)]
pub struct LoxObject {
    // TODO: This should be a reference but that's tricky
    pub class: LoxClass,
    pub vars: HashMap<String, LoxValue>,
}

#[derive(PartialEq, Clone)]
pub enum LoxValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(LoxFunction),
    Class(LoxClass),
    Object(LoxObject),
}

impl LoxValue {
    pub fn type_str(&self) -> String {
        match self {
            Self::Nil => "nil".into(),
            Self::Boolean(_) => "Boolean".into(),
            Self::Number(_) => "Number".into(),
            Self::String(_) => "String".into(),
            Self::Function(_) => "Function".into(),
            Self::Class(_) => "Class".into(),
            Self::Object(_) => "Object".into(),
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_fun(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
    
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn get_boolean(&self) -> LoxResult<bool> {
        if let Self::Boolean(value) = self {
            Ok(*value)
        } else {
            Err(LoxError::Runtime(format!(
                "Expected Boolean, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn get_number(&self) -> LoxResult<f64> {
        if let Self::Number(value) = self {
            Ok(*value)
        } else {
            Err(LoxError::Runtime(format!(
                "Expected Number, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn get_string(&self) -> LoxResult<String> {
        if let Self::String(value) = self {
            Ok(value.clone())
        } else {
            Err(LoxError::Runtime(format!(
                "Expected String, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn get_fun(&self) -> LoxResult<&LoxFunction> {
        if let Self::Function(fun) = self {
            Ok(fun)
        } else {
            Err(LoxError::Runtime(format!(
                "Expected Function, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn get_class(&self) -> LoxResult<&LoxClass> {
        if let Self::Class(class) = self {
            Ok(class)
        } else {
            Err(LoxError::Runtime(format!(
                "Expected Class, got \"{}\"",
                self.type_str()
            )))
        }
    }

    pub fn get_object(&self) -> LoxResult<&LoxObject> {
        if let Self::Object(obj) = self {
            Ok(obj)
        } else {
            Err(LoxError::Runtime(format!(
                "Expected Object, got \"{}\"",
                self.type_str()
            )))
        }
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

impl From<&str> for LoxValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<LoxFunction> for LoxValue {
    fn from(func: LoxFunction) -> Self {
        Self::Function(func)
    }
}

impl From<LoxClass> for LoxValue {
    fn from(value: LoxClass) -> Self {
        Self::Class(value)
    }
}

impl From<LoxObject> for LoxValue {
    fn from(value: LoxObject) -> Self {
        Self::Object(value)
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

impl ToString for LoxValue {
    fn to_string(&self) -> String {
        match self {
            Self::Nil => "nil".into(),
            Self::Boolean(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::String(value) => value.clone(),
            Self::Function(func) => {
                format!("<function {}>", func.name.as_ref().unwrap_or(&"".into()))
            },
            Self::Class(class) => {
                format!("<class {}>", class.name)
            },
            Self::Object(obj) => {
                format!("<instance {}>", obj.class.name)
            }
        }
    }
}
