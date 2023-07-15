use super::{error::*, globals::*, scanner::*, value::*};
use std::{collections::HashMap, rc::Rc};

pub type ScopeId = usize;

pub struct Scope {
    vars: HashMap<String, LoxValue>,
    parent: Option<ScopeId>,
    children: Vec<ScopeId>,
}

pub struct Environment {
    global: HashMap<String, LoxValue>,
    scopes: Vec<Option<Scope>>,
}

impl Environment {
    pub fn new() -> (Self, ScopeId) {
        let env = Self {
            global: globals(),
            scopes: vec![Some(Scope {
                vars: HashMap::new(),
                parent: None,
                children: vec![],
            })],
        };
        (env, 0)
    }

    pub fn new_scope(&mut self, parent: ScopeId) -> LoxResult<ScopeId> {
        match self.get_scope_mut(parent) {
            Some(parent_scope) => {
                let id = self.get_empty();
                let scope = Scope {
                    vars: HashMap::new(),
                    parent: Some(parent),
                    children: vec![],
                };
                self.scopes[id] = Some(scope);
                parent_scope.children.push(id);
                Ok(id)
            },
            None => {
                Err(LoxError::Runtime("Parent scope is missing".into()))
            }
        }
    }

    pub fn drop_scope(&mut self, scope: ScopeId) -> ScopeId {
        if let Some(parent) = self.get_parent_mut(scope) {
            self.scopes[scope] = None;
            parent.children = parent.children
                .into_iter()
                .filter(|child| *child != scope)
                .collect();
        }
        scope
    }

    pub fn get(&self, scope: ScopeId, key: &str) -> Option<&LoxValue> {
        self.get_scope(scope)
            .map(|scope| {
                scope.vars.get(key)
                    .or_else(|| {
                        match scope.parent {
                            Some(id) => self.get(id, key),
                            None => self.global.get(key)
                        }
                    })
            })
            .flatten()
    }

    pub fn get_mut(&self, scope: ScopeId, key: &str) -> Option<&mut LoxValue> {
        self.get_scope(scope)
            .map(|scope| {
                scope.vars.get_mut(key)
                    .or_else(|| {
                        match scope.parent {
                            Some(id) => self.get_mut(id, key),
                            None => self.global.get_mut(key)
                        }
                    })
            })
            .flatten()
    }

    pub fn insert(&self, scope: ScopeId, key: String, value: LoxValue) {
        if let Some(scope) = self.get_scope_mut(scope) {
            scope.vars.insert(key, value);
        }
    }

    fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        assert!(id < self.scopes.len(), "ScopeId out of range");
        self.scopes[id].as_ref()
    }

    fn get_scope_mut(&self, id: ScopeId) -> Option<&mut Scope> {
        assert!(id < self.scopes.len(), "ScopeId out of range");
        self.scopes[id].as_mut()
    }

    fn get_parent(&self, id: ScopeId) -> Option<&Scope> {
        assert!(id < self.scopes.len(), "ScopeId out of range");
        self.get_scope(id)
            .map(|scope| {
                scope.parent
                    .map(|parent| self.get_scope(parent))
                    .flatten()
            })
            .flatten()
    }

    fn get_parent_mut(&self, id: ScopeId) -> Option<&mut Scope> {
        assert!(id < self.scopes.len(), "ScopeId out of range");
        self.get_scope(id)
            .map(|scope| {
                scope.parent
                    .map(|parent| self.get_scope_mut(parent))
                    .flatten()
            })
            .flatten()
    }

    fn get_empty(&mut self) -> ScopeId {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.is_none() {
                return i;
            }
        }
        self.scopes.push(None);
        self.scopes.len() - 1
    }
}
// #[derive(PartialEq)]
// pub struct Environment {
//     globals: HashMap<String, LoxValue>,
//     locals: HashMap<String, LoxValue>,
//     parent: Option<Rc<Environment>>,
// }

// impl Environment {
//     pub fn new() -> Self {
//         Self {
//             globals: globals(),
//             locals: HashMap::new(),
//             parent: None,
//         }
//     }

//     pub fn inner(parent: Rc<Environment>) -> Self {
//         Self {
//             globals: HashMap::new(), // Child scopes don't get duplicate globals.
//             locals: HashMap::new(),
//             parent: Some(parent),
//         }
//     }

//     pub fn close(&mut self) -> LoxResult<Rc<Environment>> {
//         if self.parent.is_some() {
//             Ok(self.parent.take().unwrap())
//         } else {
//             Err(LoxError::Runtime(
//                 "Attempted to unroll top level env".into(),
//             ))
//         }
//     }

//     pub fn get(&self, name: &str) -> LoxResult<LoxValue> {
//         self.get_ref(name).map(|v| v.clone())
//     }

//     pub fn get_ref(&self, name: &str) -> LoxResult<&LoxValue> {
//         let value = self.locals.get(name).or_else(|| self.globals.get(name));
//         match value {
//             Some(value) => Ok(value),
//             None => self
//                 .parent
//                 .as_ref()
//                 .map(|parent| parent.get_ref(name))
//                 .unwrap_or(Err(LoxError::Runtime(format!(
//                     "Undefined variable '{}'",
//                     name
//                 )))),
//         }
//     }

//     pub fn _get_mut(&mut self, name: &str) -> LoxResult<&mut LoxValue> {
//         let value = self.locals.get_mut(name);
//         match value {
//             Some(value) => Ok(value),
//             None => self
//                 .parent
//                 .as_mut()
//                 .map(|parent| Rc::get_mut(parent).unwrap()._get_mut(name))
//                 .unwrap_or(Err(LoxError::Runtime(format!(
//                     "Undefined variable '{}'",
//                     name
//                 )))),
//         }
//     }

//     pub fn get_by_token(&self, identifier: &Token) -> LoxResult<LoxValue> {
//         assert!(matches!(identifier.kind, TokenKind::Identifier));
//         self.get(&identifier.lexeme_str())
//     }

//     pub fn declare(&mut self, name: &str, value: LoxValue) {
//         self.locals.insert(name.into(), value);
//     }

//     pub fn declare_by_token(&mut self, identifier: &Token, value: LoxValue) {
//         assert!(matches!(identifier.kind, TokenKind::Identifier));
//         self.declare(&identifier.lexeme_str(), value);
//     }

//     pub fn assign(&mut self, name: &str, value: LoxValue) -> LoxResult<Option<LoxValue>> {
//         if self.locals.contains_key(name) {
//             Ok(self.locals.insert(name.into(), value))
//         } else if let Some(parent) = &mut self.parent {
//             Rc::get_mut(parent).unwrap().assign(name, value)
//         } else {
//             Err(LoxError::Runtime(format!(
//                 "Attempted to assign unbound variable \"{}\"",
//                 name
//             )))
//         }
//     }

//     pub fn assign_by_token(
//         &mut self,
//         identifier: &Token,
//         value: LoxValue,
//     ) -> LoxResult<Option<LoxValue>> {
//         assert!(matches!(identifier.kind, TokenKind::Identifier));
//         self.assign(&identifier.lexeme_str(), value)
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn declare() {
//         let mut env = Environment::new();
//         env.declare("foo", LoxValue::String("bar".into()));
//         assert!(env.get("foo").unwrap() == LoxValue::String("bar".into()));
//     }

//     #[test]
//     fn token_declare() {
//         let mut env = Environment::new();
//         let identifier = Token::new(TokenKind::Identifier, Some("foo".into()), None, 0);
//         env.declare_by_token(&identifier, LoxValue::String("bar".into()));
//         assert!(env.get_by_token(&identifier).unwrap() == LoxValue::String("bar".into()));
//     }

//     #[test]
//     fn assign() {
//         let mut env = Environment::new();
//         assert!(env.assign("foo", LoxValue::Nil).is_err());
//         env.declare("foo", LoxValue::String("foo".into()));
//         assert!(
//             env.assign("foo", LoxValue::String("bar".into()))
//                 .unwrap()
//                 .unwrap()
//                 == LoxValue::String("foo".into())
//         );
//     }

//     #[test]
//     fn token_assign() {
//         let mut env = Environment::new();
//         let identifier = Token::new(TokenKind::Identifier, Some("foo".into()), None, 0);
//         assert!(env.assign_by_token(&identifier, LoxValue::Nil).is_err());
//         env.declare_by_token(&identifier, LoxValue::String("foo".into()));
//         assert!(
//             env.assign_by_token(&identifier, LoxValue::String("bar".into()))
//                 .unwrap()
//                 .unwrap()
//                 == LoxValue::String("foo".into())
//         );
//     }

//     #[test]
//     fn parents() {
//         let mut env = Rc::new(Environment::new());
//         Rc::get_mut(&mut env)
//             .unwrap()
//             .declare("level", LoxValue::String("one".into()));
//         Rc::get_mut(&mut env)
//             .unwrap()
//             .declare("global", LoxValue::Number(1.0));
//         env = Rc::new(Environment::inner(env));
//         Rc::get_mut(&mut env)
//             .unwrap()
//             .declare("level", LoxValue::String("two".into()));
//         env = Rc::new(Environment::inner(env));
//         Rc::get_mut(&mut env)
//             .unwrap()
//             .declare("level", LoxValue::String("three".into()));
//         Rc::get_mut(&mut env)
//             .unwrap()
//             .assign("global", LoxValue::Number(2.0))
//             .unwrap();
//         assert!(env.get("level").unwrap() == LoxValue::String("three".into()));
//         assert!(env.get("global").unwrap() == LoxValue::Number(2.0));
//         env = Rc::get_mut(&mut env).unwrap().close().unwrap();
//         assert!(env.get("level").unwrap() == LoxValue::String("two".into()));
//         env = Rc::get_mut(&mut env).unwrap().close().unwrap();
//         assert!(env.get("level").unwrap() == LoxValue::String("one".into()));
//     }
// }
