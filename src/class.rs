use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    environment::LoxProperties, error::LoxResult, object::*, state::LoxState, value::LoxValue,
};

use super::function::*;

#[derive(PartialEq, Clone)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Rc<RefCell<LoxClass>>>,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    /// Intended to be used from builtins. Does not look up super classes
    pub fn instantiate(&self, state: &mut LoxState, args: &[LoxValue]) -> LoxResult<LoxValue> {
        let obj = Rc::new(RefCell::new(LoxObject {
            class_name: self.name.clone(),
            props: LoxProperties::new(),
        }));
        let this_value = LoxValue::from(obj.clone());
        for (name, func) in self.methods.iter() {
            let mut method = func.clone();
            method.this_value = Some(this_value.clone());
            obj.borrow_mut().props.insert(name.clone(), method.into());
        }
        let init = {
            obj.borrow()
                .props
                .get("init")
                .and_then(|init| init.get_fun().ok())
        };
        if let Some(init) = init {
            init.borrow().call_native(state, args)?;
        }
        Ok(obj.into())
    }
}
