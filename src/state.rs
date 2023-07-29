use crate::{
    environment::{ScopeHandle, GLOBAL_SCOPE},
    error::{LoxError, LoxResult},
    expr::Expr,
};

use super::{environment::Environment, resolver::Locals, value::LoxValue};

pub struct LoxState {
    pub env: Environment,
    pub locals: Locals,
    pub stack: Vec<LoxValue>,
}

impl LoxState {
    pub fn new(locals: Locals) -> Self {
        Self {
            env: Environment::new(),
            locals,
            stack: vec![],
        }
    }

    pub fn resolve_local(&self, scope: ScopeHandle, expr: &Expr, key: &str) -> LoxResult<LoxValue> {
        let scope = match self.locals.get(&expr.id()) {
            Some(depth) => self
                .env
                .ancestor_scope(scope, *depth)
                .ok_or_else(|| LoxError::Runtime("Invalid scope".into())),
            None => Ok(GLOBAL_SCOPE),
        }?;
        self.env
            .get(Some(scope), key)
            .ok_or_else(|| LoxError::Runtime(format!("Undefined variable \"{}\"", key)))
    }
}
