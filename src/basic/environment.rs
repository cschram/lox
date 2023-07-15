use super::{error::*, globals::*, scanner::*, value::*};
use std::{collections::HashMap, rc::Rc};

pub struct Environment {
    globals: HashMap<String, LoxValue>,
    locals: HashMap<String, LoxValue>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: globals(),
            locals: HashMap::new(),
            parent: None,
        }
    }

    pub fn inner(parent: Rc<Environment>) -> Self {
        Self {
            globals: HashMap::new(), // Child scopes don't get duplicate globals.
            locals: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn close(&mut self) -> LoxResult<Rc<Environment>> {
        if self.parent.is_some() {
            Ok(self.parent.take().unwrap())
        } else {
            Err(LoxError::Runtime(
                "Attempted to unroll top level env".into(),
            ))
        }
    }

    pub fn get(&self, name: &str) -> LoxResult<LoxValue> {
        self.get_ref(name).map(|v| v.clone())
    }

    pub fn get_ref(&self, name: &str) -> LoxResult<&LoxValue> {
        let value = self.locals.get(name).or_else(|| self.globals.get(name));
        match value {
            Some(value) => Ok(value),
            None => self
                .parent
                .as_ref()
                .map(|parent| parent.get_ref(name))
                .unwrap_or(Err(LoxError::Runtime(format!(
                    "Undefined variable '{}'",
                    name
                )))),
        }
    }

    pub fn _get_mut(&mut self, name: &str) -> LoxResult<&mut LoxValue> {
        let value = self.locals.get_mut(name);
        match value {
            Some(value) => Ok(value),
            None => self
                .parent
                .as_mut()
                .map(|parent| Rc::get_mut(parent).unwrap()._get_mut(name))
                .unwrap_or(Err(LoxError::Runtime(format!(
                    "Undefined variable '{}'",
                    name
                )))),
        }
    }

    pub fn get_by_token(&self, identifier: &Token) -> LoxResult<LoxValue> {
        assert!(matches!(identifier.kind, TokenKind::Identifier));
        self.get(&identifier.lexeme_str())
    }

    pub fn declare(&mut self, name: &str, value: LoxValue) {
        self.locals.insert(name.into(), value);
    }

    pub fn declare_by_token(&mut self, identifier: &Token, value: LoxValue) {
        assert!(matches!(identifier.kind, TokenKind::Identifier));
        self.declare(&identifier.lexeme_str(), value);
    }

    pub fn assign(&mut self, name: &str, value: LoxValue) -> LoxResult<Option<LoxValue>> {
        if self.locals.contains_key(name) {
            Ok(self.locals.insert(name.into(), value))
        } else if let Some(parent) = &mut self.parent {
            Rc::get_mut(parent).unwrap().assign(name, value)
        } else {
            Err(LoxError::Runtime(format!(
                "Attempted to assign unbound variable \"{}\"",
                name
            )))
        }
    }

    pub fn assign_by_token(
        &mut self,
        identifier: &Token,
        value: LoxValue,
    ) -> LoxResult<Option<LoxValue>> {
        assert!(matches!(identifier.kind, TokenKind::Identifier));
        self.assign(&identifier.lexeme_str(), value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn declare() {
        let mut env = Environment::new();
        env.declare("foo", LoxValue::String("bar".into()));
        assert!(env.get("foo").unwrap() == LoxValue::String("bar".into()));
    }

    #[test]
    fn token_declare() {
        let mut env = Environment::new();
        let identifier = Token::new(TokenKind::Identifier, Some("foo".into()), None, 0);
        env.declare_by_token(&identifier, LoxValue::String("bar".into()));
        assert!(env.get_by_token(&identifier).unwrap() == LoxValue::String("bar".into()));
    }

    #[test]
    fn assign() {
        let mut env = Environment::new();
        assert!(env.assign("foo", LoxValue::Nil).is_err());
        env.declare("foo", LoxValue::String("foo".into()));
        assert!(
            env.assign("foo", LoxValue::String("bar".into()))
                .unwrap()
                .unwrap()
                == LoxValue::String("foo".into())
        );
    }

    #[test]
    fn token_assign() {
        let mut env = Environment::new();
        let identifier = Token::new(TokenKind::Identifier, Some("foo".into()), None, 0);
        assert!(env.assign_by_token(&identifier, LoxValue::Nil).is_err());
        env.declare_by_token(&identifier, LoxValue::String("foo".into()));
        assert!(
            env.assign_by_token(&identifier, LoxValue::String("bar".into()))
                .unwrap()
                .unwrap()
                == LoxValue::String("foo".into())
        );
    }

    #[test]
    fn parents() {
        let mut env = Rc::new(Environment::new());
        Rc::get_mut(&mut env)
            .unwrap()
            .declare("level", LoxValue::String("one".into()));
        Rc::get_mut(&mut env)
            .unwrap()
            .declare("global", LoxValue::Number(1.0));
        env = Rc::new(Environment::inner(env));
        Rc::get_mut(&mut env)
            .unwrap()
            .declare("level", LoxValue::String("two".into()));
        env = Rc::new(Environment::inner(env));
        Rc::get_mut(&mut env)
            .unwrap()
            .declare("level", LoxValue::String("three".into()));
        Rc::get_mut(&mut env)
            .unwrap()
            .assign("global", LoxValue::Number(2.0))
            .unwrap();
        assert!(env.get("level").unwrap() == LoxValue::String("three".into()));
        assert!(env.get("global").unwrap() == LoxValue::Number(2.0));
        env = Rc::get_mut(&mut env).unwrap().close().unwrap();
        assert!(env.get("level").unwrap() == LoxValue::String("two".into()));
        env = Rc::get_mut(&mut env).unwrap().close().unwrap();
        assert!(env.get("level").unwrap() == LoxValue::String("one".into()));
    }
}
