use super::{class::*, environment::*, error::*, function::*, value::*};
use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn get_builtins() -> LoxProperties {
    let mut constants = LoxProperties::new();

    let class_vec = LoxClass {
        name: "List".into(),
        superclass: None,
        methods: {
            let init = LoxFunction::native("init", vec![], |_, this_value, _| {
                let this = this_value.expect("Expected a this value").get_object()?;
                this.borrow_mut().set("__vec__".into(), Vec::<LoxValue>::new().into());
                Ok(LoxValue::Nil)
            });

            let method_get = LoxFunction::native("get", vec!["index"], |_, this_value, args| {
                if args.is_empty() {
                    return Err(LoxError::Runtime("Expected 1 argument".into()));
                }
                let index = args[0].get_number()? as usize;
                let this = this_value.expect("Expected a this value").get_object()?;
                let __vec__ = this.borrow().get("__vec__").expect("Missing __vec__").get_vec()?;
                if index > __vec__.borrow().len() {
                    return Err(LoxError::Runtime(format!("Index {index} out of range")));
                }
                let elem = &__vec__.borrow()[index];
                Ok(elem.clone())
            });

            let method_set = LoxFunction::native("get", vec!["index", "value"], |_, this_value, args| {
                if args.len() < 2 {
                    return Err(LoxError::Runtime("Expected 2 arguments".into()));
                }
                let index = args[0].get_number()? as usize;
                let this = this_value.expect("Expected a this value").get_object()?;
                let __vec__ = this.borrow().get("__vec__").expect("Missing __vec__").get_vec()?;
                if index > __vec__.borrow().len() {
                    return Err(LoxError::Runtime(format!("Index {index} out of range")));
                }
                __vec__.borrow_mut()[index] = args[1].clone();
                Ok(LoxValue::Nil)
            });

            let method_push = LoxFunction::native("get", vec!["value"], |_, this_value, args| {
                if args.is_empty() {
                    return Err(LoxError::Runtime("Expected 1 argument".into()));
                }
                let this = this_value.expect("Expected a this value").get_object()?;
                let __vec__ = this.borrow().get("__vec__").expect("Missing __vec__").get_vec()?;
                __vec__.borrow_mut().push(args[1].clone());
                Ok(LoxValue::Nil)
            });

            let method_pop = LoxFunction::native("get", vec![], |_, this_value, _| {
                let this = this_value.expect("Expected a this value").get_object()?;
                let __vec__ = this.borrow().get("__vec__").expect("Missing __vec__").get_vec()?;
                let value = __vec__.borrow_mut().pop();
                Ok(value.unwrap_or(LoxValue::Nil))
            });

            let mut methods = HashMap::<String, LoxFunction>::new();
            methods.insert("init".into(), init);
            methods.insert("get".into(), method_get);
            methods.insert("set".into(), method_set);
            methods.insert("push".into(), method_push);
            methods.insert("pop".into(), method_pop);
            methods
        }
    };
    
    constants.insert("Vec".into(), class_vec.into());

    let func_time = LoxFunction::native("time", vec![], |_, _, _| {
        let now = SystemTime::now();
        let elapsed = now.duration_since(UNIX_EPOCH)?;
        Ok(LoxValue::Number(elapsed.as_millis() as f64))
    });

    constants.insert("time".into(), func_time.into());

    let func_args = LoxFunction::native("args", vec![], |state, _, _| {
        let args: Vec<LoxValue> = env::args().map(|arg| LoxValue::from(arg)).collect();
        let class_vec = state.env.get(None, "Vec").expect("Expected Vec to exist").get_class()?;
        let lox_vec = class_vec.borrow().instantiate(state, &[])?;
        lox_vec.get_object()?.borrow_mut().set("__vec__".into(), args.into());
        Ok(lox_vec)
    });

    constants.insert("args".into(), func_args.into());

    constants
}
