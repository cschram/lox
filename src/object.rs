use super::{class::*, environment::*, error::*, expr::*, state::*, value::*};
use std::{cell::RefCell, rc::Rc};

#[derive(PartialEq, Clone)]
pub struct LoxProperties {
    props: LoxVars,
    superprops: Option<Box<LoxProperties>>,
}

impl LoxProperties {
    fn bind(&mut self, class: Rc<RefCell<LoxClass>>) {
        if let Some(superclass) = class.borrow().superclass {
            let mut superprops = LoxProperties {
                props: LoxVars::new(),
                superprops: None,
            };
            superprops.bind(superclass.clone());
            self.superprops = Some(Box::new(superprops));
        }
        class.borrow().bind(obj);
    }

    pub fn get(&self, key: &str) -> Option<LoxValue> {
        self.props.get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: LoxValue) -> Option<LoxValue> {
        self.props.insert(key, value)
    }

    pub fn get_super(&self, key: &str) -> Option<LoxValue> {
        self.superprops.and_then(|superprops| superprops.get(key))
    }
}

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
    ) -> LoxResult<Self> {
        let mut obj = Self {
            class: class.clone(),
            props: LoxProperties {
                props: LoxVars::new(),
                superprops: None,
            },
        };
        obj.props.bind(class.clone());
        let init = { obj.props.get("init").and_then(|init| init.get_fun().ok()) };
        if let Some(init) = init {
            println!("init");
            init.borrow().call(state, scope, arguments)?;
        }
        Ok(obj)
    }
}
