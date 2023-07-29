use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{environment::*, function::*, value::*};

#[derive(PartialEq, Clone)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Rc<RefCell<LoxClass>>>,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn bind_methods(&self, props: &mut LoxProperties, this_value: LoxValue, super_value: Option<LoxValue>) {
        for (name, fun) in self.methods.iter() {
            let mut method = fun.clone();
            if matches!(&method.name, Some(name) if name == "init") {
                method.is_constructor = true;
            }
            method.this_value = Some(this_value.clone());
            method.super_value = super_value.clone();
            props.insert(name.clone(), method.into());
        }
    }
}
