use super::{builtins::*, value::*};
use std::collections::HashMap;

#[derive(PartialEq, Clone, Copy)]
pub struct ScopeHandle(usize);

pub struct Scope {
    vars: HashMap<String, LoxValue>,
    parent: Option<ScopeHandle>,
    children: Vec<ScopeHandle>,
}

pub struct Environment {
    globals: HashMap<String, LoxValue>,
    scopes: Vec<Option<Scope>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            scopes: vec![],
        }
    }

    pub fn get_global(&self, key: String) -> Option<LoxValue> {
        self.globals.get(&key)
            .or_else(|| BUILTINS.get(&key))
            .map(|value| value.clone())
    }

    pub fn new_scope(&mut self, parent: Option<ScopeHandle>) -> ScopeHandle {
        let id = self.get_empty();
        let scope = Scope {
            vars: HashMap::new(),
            parent,
            children: vec![],
        };
        self.scopes[id.0] = Some(scope);
        if let Some(id) = parent {
            self.get_scope_mut(id).expect("Invalid scope").children.push(id);
        }
        id
    }

    pub fn drop_scope(&mut self, handle: ScopeHandle) {
        let scope = self.get_scope(handle).expect("Invalid scope");
        for child in scope.children.clone().iter() {
            self.drop_scope(*child);
        }
        if let Some(parent) = self.get_parent_mut(handle) {
            parent.children.retain(|child| *child != handle);
        }
        self.scopes[handle.0] = None;
    }

    pub fn parent_scope(&self, handle: ScopeHandle) -> Option<ScopeHandle> {
        self.get_scope(handle)
            .and_then(|scope| scope.parent)
    }

    pub fn ancestor_scope(&self, handle: ScopeHandle, distance: u128) -> Option<ScopeHandle> {
        if distance > 0 {
            let parent = self.parent_scope(handle)?;
            self.ancestor_scope(parent, distance - 1)
        } else {
            Some(handle)
        }
    }

    pub fn root_scope(&self, handle: ScopeHandle) -> ScopeHandle {
        match self.parent_scope(handle) {
            Some(parent) => {
                self.root_scope(parent)
            },
            None => handle
        }
    }

    // TODO: Value vs Reference semantics
    pub fn get(&self, handle: ScopeHandle, key: &str) -> Option<LoxValue> {
        let scope = self.get_scope(handle)?;
        scope.vars.get(key)
            .map(|value| value.clone())
            .or_else(|| {
                match scope.parent {
                    Some(parent) => {
                        self.get(parent, key)
                    },
                    None => self.get_builtin(key)
                }
            })
    }

    pub fn get_from(&self, handle: ScopeHandle, key: &str) -> Option<LoxValue> {
        let scope = self.get_scope(handle)?;
        scope.vars.get(key)
            .map(|value| value.clone())
            .or_else(|| self.get_builtin(key))
    }

    pub fn insert(&mut self, handle: ScopeHandle, key: String, value: LoxValue) {
        if let Some(scope) = self.get_scope_mut(handle) {
            scope.vars.insert(key, value);
        }
    }

    pub fn assign(&mut self, handle: ScopeHandle, key: String, value: LoxValue) -> Option<LoxValue> {
        let scope = self.get_scope_mut(handle).expect("Invalid scope");
        if scope.vars.contains_key(&key) {
            scope.vars.insert(key, value)
        } else {
            scope.parent.and_then(|parent| {
                self.assign(parent, key, value)
            })
        }
    }

    fn get_scope(&self, handle: ScopeHandle) -> Option<&Scope> {
        assert!(handle.0 < self.scopes.len(), "ScopeId out of range");
        self.scopes[handle.0].as_ref()
    }

    fn get_scope_mut(&mut self, handle: ScopeHandle) -> Option<&mut Scope> {
        assert!(handle.0 < self.scopes.len(), "ScopeId out of range");
        self.scopes[handle.0].as_mut()
    }

    fn get_parent_mut(&mut self, handle: ScopeHandle) -> Option<&mut Scope> {
        assert!(handle.0 < self.scopes.len(), "ScopeId out of range");
        self.get_scope_mut(self.get_scope(handle)?.parent?)
    }

    // TODO: Don't clone everywhere
    fn get_builtin(&self, key: &str) -> Option<LoxValue> {
        BUILTINS.get(key).map(|value| value.clone())
    }

    fn get_empty(&mut self) -> ScopeHandle {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.is_none() {
                return ScopeHandle(i);
            }
        }
        self.scopes.push(None);
        ScopeHandle(self.scopes.len() - 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut env = Environment::new();
        let scope = env.new_scope(None);
        env.insert(scope, "foo".into(), "one".into());
        assert!(env.get(scope, "foo").unwrap() == "one".into());
        env.insert(scope, "foo".into(), "two".into());
        assert!(env.get(scope, "foo").unwrap() == "two".into());
    }

    #[test]
    fn nested() {
        let mut env = Environment::new();
        let outer_scope = env.new_scope(None);
        env.insert(outer_scope, "foo".into(), "one".into());
        env.insert(outer_scope, "bar".into(), "two".into());
        let inner_scope = env.new_scope(Some(outer_scope));
        env.insert(inner_scope, "foo".into(), "three".into());
        assert!(env.get(inner_scope, "foo").unwrap() == "three".into());
        assert!(env.get(inner_scope, "bar").unwrap() == "two".into());
    }

    #[test]
    fn ancestors() {
        let mut env = Environment::new();
        let one = env.new_scope(None);
        let two = env.new_scope(Some(one));
        let three = env.new_scope(Some(two));
        assert!(env.ancestor_scope(three, 2).unwrap() == one);
    }
}
