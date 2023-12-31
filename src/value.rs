use std::{cell::RefCell, rc::Rc};

use crate::environment::LoxProperties;

use super::{class::*, error::*, function::*, object::*, scanner::*};

#[derive(PartialEq, Clone)]
pub enum LoxValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Rc<RefCell<LoxFunction>>),
    Class(Rc<RefCell<LoxClass>>),
    Object(Rc<RefCell<LoxObject>>),
    Super(Rc<LoxProperties>),
    Vec(Rc<RefCell<Vec<LoxValue>>>),
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
            Self::Super(_) => "Super".into(),
            Self::Vec(_) => "Vec".into(),
        }
    }

    #[allow(dead_code)]
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    #[allow(dead_code)]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[allow(dead_code)]
    pub fn is_fun(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    #[allow(dead_code)]
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }

    #[allow(dead_code)]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    #[allow(dead_code)]
    pub fn is_super(&self) -> bool {
        matches!(self, Self::Super(_))
    }

    #[allow(dead_code)]
    pub fn get_boolean(&self, line: u32) -> LoxResult<bool> {
        if let Self::Boolean(value) = self {
            Ok(*value)
        } else {
            Err(LoxError::Runtime(
                format!("Expected Boolean, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    pub fn get_number(&self, line: u32) -> LoxResult<f64> {
        if let Self::Number(value) = self {
            Ok(*value)
        } else {
            Err(LoxError::Runtime(
                format!("Expected Number, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    #[allow(dead_code)]
    pub fn get_string(&self, line: u32) -> LoxResult<String> {
        if let Self::String(value) = self {
            Ok(value.clone())
        } else {
            Err(LoxError::Runtime(
                format!("Expected String, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    pub fn get_fun(&self, line: u32) -> LoxResult<Rc<RefCell<LoxFunction>>> {
        if let Self::Function(fun) = self {
            Ok(fun.clone())
        } else {
            Err(LoxError::Runtime(
                format!("Expected Function, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    pub fn get_class(&self, line: u32) -> LoxResult<Rc<RefCell<LoxClass>>> {
        if let Self::Class(class) = self {
            Ok(class.clone())
        } else {
            Err(LoxError::Runtime(
                format!("Expected Class, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    pub fn get_object(&self, line: u32) -> LoxResult<Rc<RefCell<LoxObject>>> {
        if let Self::Object(obj) = self {
            Ok(obj.clone())
        } else {
            Err(LoxError::Runtime(
                format!("Expected Object, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    pub fn get_super(&self, line: u32) -> LoxResult<Rc<LoxProperties>> {
        if let Self::Super(methods) = self {
            Ok(methods.clone())
        } else {
            Err(LoxError::Runtime(
                format!("Expected Super, got \"{}\"", self.type_str()),
                line,
            ))
        }
    }

    pub fn get_vec(&self, line: u32) -> LoxResult<Rc<RefCell<Vec<LoxValue>>>> {
        if let Self::Vec(vec) = self {
            Ok(vec.clone())
        } else {
            Err(LoxError::Runtime(
                format!("Expected Vec, got \"{}\"", self.type_str()),
                line,
            ))
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
    fn from(value: LoxFunction) -> Self {
        Self::Function(Rc::new(RefCell::new(value)))
    }
}

impl From<LoxClass> for LoxValue {
    fn from(value: LoxClass) -> Self {
        Self::Class(Rc::new(RefCell::new(value)))
    }
}

impl From<LoxObject> for LoxValue {
    fn from(value: LoxObject) -> Self {
        Self::Object(Rc::new(RefCell::new(value)))
    }
}

impl From<Rc<RefCell<LoxObject>>> for LoxValue {
    fn from(value: Rc<RefCell<LoxObject>>) -> Self {
        Self::Object(value)
    }
}

impl From<Rc<LoxProperties>> for LoxValue {
    fn from(value: Rc<LoxProperties>) -> Self {
        Self::Super(value)
    }
}

impl From<Vec<LoxValue>> for LoxValue {
    fn from(value: Vec<LoxValue>) -> Self {
        Self::Vec(Rc::new(RefCell::new(value)))
    }
}

impl From<Rc<RefCell<Vec<LoxValue>>>> for LoxValue {
    fn from(value: Rc<RefCell<Vec<LoxValue>>>) -> Self {
        Self::Vec(value)
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
                format!(
                    "<function {}>",
                    func.borrow().name.as_ref().unwrap_or(&"".into())
                )
            }
            Self::Class(class) => {
                format!("<class {}>", class.borrow().name)
            }
            Self::Object(obj) => {
                format!("<instance {}>", obj.borrow().class_name)
            }
            Self::Super(_) => "<super>".into(),
            Self::Vec(_) => "<vec>".into(),
        }
    }
}
