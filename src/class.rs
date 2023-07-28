use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{function::*, object::*};

#[derive(PartialEq, Clone)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Rc<RefCell<LoxClass>>>,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn bind(&self, obj: &mut LoxObject) {
        for (name, fun) in self.methods.iter() {
            let mut method = fun.clone();
            if matches!(&method.name, Some(name) if name == "init") {
                method.is_constructor = true;
            }
            method.this = Some(obj.clone().into());
            obj.vars.insert(name.clone(), method.into());
        }
    }
}
