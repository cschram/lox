use super::{class::*, environment::*, error::*, expr::*, state::*, value::*};
use std::{cell::RefCell, rc::Rc};

#[derive(PartialEq, Clone)]
pub struct LoxObject {
    pub class: Rc<RefCell<LoxClass>>,
    pub props: LoxVars,
    pub methods: Vec<LoxVars>,
}

impl LoxObject {
    pub fn instantiate(
        class: Rc<RefCell<LoxClass>>,
        state: &mut LoxState,
        scope: ScopeHandle,
        arguments: &[Expr],
    ) -> LoxResult<LoxValue> {
        let mut obj = Rc::new(RefCell::new(Self {
            class: class.clone(),
            props: LoxVars::new(),
            methods: vec![],
        }));
        let this_value = LoxValue::from(obj.clone());
        // let mut current_class = Some(class.clone());
        // while let Some(class) = &current_class {
        //     let mut props = LoxVars::new();
        //     let super_value = { class.borrow().superclass.clone() };
        //     class.borrow().bind_methods(&mut props, this_value, super_value);
        // }
        let init = {
            obj.borrow().props.get("init").and_then(|init| init.get_fun().ok()) };
        if let Some(init) = init {
            init.borrow().call(state, scope, arguments)?;
        }
        Ok(this_value)
    }

    pub fn has(&self, key: &str) -> bool {
        self.props.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<LoxValue> {
        self.props.get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: LoxValue) -> Option<LoxValue> {
        self.props.insert(key, value)
    }
}
