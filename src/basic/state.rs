use super::{
    environment::Environment,
    resolver::Locals,
    value::LoxValue,
};

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
}
