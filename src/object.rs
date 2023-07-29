use super::{class::*, environment::*, error::*, expr::*, state::*, value::*};
use std::{cell::RefCell, rc::Rc};

#[derive(PartialEq, Clone)]
pub struct LoxObject {
    pub class: Rc<RefCell<LoxClass>>,
    pub props: LoxProperties,
}

impl LoxObject {
    pub fn instantiate(
        class: Rc<RefCell<LoxClass>>,
        state: &mut LoxState,
        scope: ScopeHandle,
        arguments: &[Expr],
    ) -> LoxResult<LoxValue> {
        let obj = Rc::new(RefCell::new(Self {
            class: class.clone(),
            props: LoxProperties::new(),
        }));
        let this_value = LoxValue::from(obj.clone());

        let classes: Vec<Rc<RefCell<LoxClass>>> = {
            let mut classes: Vec<Rc<RefCell<LoxClass>>> = vec![];
            let mut current_class = Some(class.clone());
            while let Some(class) = current_class {
                classes.push(class.clone());
                current_class = class.borrow().superclass.clone();
            }
            classes.into_iter().rev().collect()
        };

        {
            let mut super_value: Option<Rc<LoxProperties>> = None;
            for class in classes.into_iter() {
                let mut super_methods = LoxProperties::new();
                for (name, func) in class.borrow().methods.iter() {
                    let mut method = func.clone();
                    method.this_value = Some(this_value.clone());
                    method.super_value = super_value
                        .as_ref()
                        .map(|value| LoxValue::from(value.clone()));
                    let method_value = LoxValue::from(method);
                    super_methods.insert(name.clone(), method_value.clone());
                    obj.borrow_mut().props.insert(name.clone(), method_value.clone());
                }
                super_value = Some(Rc::new(super_methods));
            }
        };

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
