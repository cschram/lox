use super::{builtins::*, value::*};
use std::collections::HashMap;

pub type LoxVars = HashMap<String, LoxValue>;

#[derive(PartialEq, Clone, Copy)]
pub struct ScopeHandle(usize);

impl std::fmt::Display for ScopeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScopeHandle({})", self.0)
    }
}

pub const GLOBAL_SCOPE: ScopeHandle = ScopeHandle(0);

pub struct Scope {
    vars: LoxVars,
    parent: Option<ScopeHandle>,
    children: Vec<ScopeHandle>,
}

pub struct Environment {
    builtins: LoxVars,
    scopes: Vec<Option<Scope>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            builtins: get_builtins(),
            scopes: vec![
                // Root scope
                Some(Scope {
                    vars: HashMap::new(),
                    parent: None,
                    children: vec![],
                }),
            ],
        }
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
            self.get_scope_mut(id)
                .expect("Invalid scope")
                .children
                .push(id);
        }
        id
    }

    pub fn parent_scope(&self, handle: ScopeHandle) -> Option<ScopeHandle> {
        self.get_scope(handle).and_then(|scope| scope.parent)
    }

    pub fn ancestor_scope(&self, handle: ScopeHandle, distance: usize) -> Option<ScopeHandle> {
        if distance > 0 {
            let parent = self.parent_scope(handle)?;
            self.ancestor_scope(parent, distance - 1)
        } else {
            Some(handle)
        }
    }

    pub fn get(&self, handle: Option<ScopeHandle>, key: &str) -> Option<LoxValue> {
        let scope = self.get_scope(handle.unwrap_or(GLOBAL_SCOPE))?;
        scope
            .vars
            .get(key)
            .cloned()
            .or_else(|| self.get_builtin(key))
    }

    pub fn declare(&mut self, handle: Option<ScopeHandle>, key: String, value: LoxValue) {
        if let Some(scope) = self.get_scope_mut(handle.unwrap_or(GLOBAL_SCOPE)) {
            scope.vars.insert(key, value);
        }
    }

    pub fn assign(
        &mut self,
        handle: Option<ScopeHandle>,
        key: String,
        value: LoxValue,
    ) -> Option<LoxValue> {
        let scope = self
            .get_scope_mut(handle.unwrap_or(GLOBAL_SCOPE))
            .expect("Invalid scope");
        assert!(
            scope.vars.contains_key(&key),
            "Cannot assign variable before declaration"
        );
        scope.vars.insert(key, value)
    }

    fn get_scope(&self, handle: ScopeHandle) -> Option<&Scope> {
        assert!(handle.0 < self.scopes.len(), "ScopeId out of range");
        self.scopes[handle.0].as_ref()
    }

    fn get_scope_mut(&mut self, handle: ScopeHandle) -> Option<&mut Scope> {
        assert!(handle.0 < self.scopes.len(), "ScopeId out of range");
        self.scopes[handle.0].as_mut()
    }

    // TODO: Don't clone everywhere
    fn get_builtin(&self, key: &str) -> Option<LoxValue> {
        self.builtins.get(key).cloned()
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
        env.declare(None, "foo".into(), "one".into());
        assert!(env.get(None, "foo").unwrap() == "one".into());
        env.declare(None, "foo".into(), "two".into());
        assert!(env.get(None, "foo").unwrap() == "two".into());
    }

    #[test]
    fn nested() {
        let mut env = Environment::new();
        env.declare(None, "foo".into(), "one".into());
        let inner_scope = Some(env.new_scope(None));
        env.declare(inner_scope, "foo".into(), "three".into());
        assert!(env.get(inner_scope, "foo").unwrap() == "three".into());
    }

    #[test]
    fn ancestors() {
        let mut env = Environment::new();
        env.declare(None, "foo".into(), "global".into());
        let one = env.new_scope(None);
        env.declare(Some(one), "foo".into(), "one".into());
        let two = env.new_scope(Some(one));
        env.declare(Some(two), "foo".into(), "two".into());
        let three = env.new_scope(Some(two));
        env.declare(Some(three), "foo".into(), "three".into());
        assert!(env.ancestor_scope(three, 2).unwrap() == one);
        assert!(env.get(env.ancestor_scope(three, 2), "foo") == Some("one".into()));
    }
}
