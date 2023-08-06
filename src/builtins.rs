use super::{class::*, environment::*, error::*, function::*, value::*};
use std::{
    collections::HashMap,
    env,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn get_builtins() -> LoxProperties {
    let mut constants = LoxProperties::new();

    let class_array = LoxClass {
        name: "Array".into(),
        superclass: None,
        methods: {
            let init = LoxFunction::native("init", vec![], |_, this_value, _| {
                let this = this_value.expect("Expected a this value").get_object()?;
                this.borrow_mut().set("__vec__".into(), Vec::<LoxValue>::new().into());
                Ok(LoxValue::Nil)
            });

            let method_len = LoxFunction::native("len", vec![], |_, this_value, _| {
                let this = this_value.expect("Expected a this value").get_object()?;
                let __vec__ = this.borrow().get("__vec__").expect("Missing __vec__").get_vec()?;
                let len = __vec__.borrow().len() as f64;
                Ok(len.into())
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

            let method_set = LoxFunction::native("set", vec!["index", "value"], |_, this_value, args| {
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
                __vec__.borrow_mut().push(args[0].clone());
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
            methods.insert("len".into(), method_len);
            methods.insert("get".into(), method_get);
            methods.insert("set".into(), method_set);
            methods.insert("push".into(), method_push);
            methods.insert("pop".into(), method_pop);
            methods
        }
    };
    
    constants.insert("Array".into(), class_array.into());

    let func_time = LoxFunction::native("time", vec![], |_, _, _| {
        let now = SystemTime::now();
        let elapsed = now.duration_since(UNIX_EPOCH)?;
        Ok(LoxValue::Number(elapsed.as_millis() as f64))
    });

    constants.insert("time".into(), func_time.into());

    let func_get_args = LoxFunction::native("get_args", vec![], |state, _, _| {
        let args: Vec<LoxValue> = env::args().map(|arg| LoxValue::from(arg)).collect();
        let class_vec = state.env.get(None, "Array").expect("Expected Array to exist").get_class()?;
        let lox_vec = class_vec.borrow().instantiate(state, &[])?;
        lox_vec.get_object()?.borrow_mut().set("__vec__".into(), args.into());
        Ok(lox_vec)
    });

    constants.insert("get_args".into(), func_get_args.into());

    constants
}

#[cfg(test)]
mod test {
    use mock_logger::MockLogger;
    use crate::{
        error::*,
        interpreter::*,
    };

   #[test]
    fn array() -> LoxResult {
        mock_logger::init();
        let mut lox = LoxInterpreter::new();
        lox.exec(r#"
            var arr = Array();
            arr.push(1);
            arr.push(2);
            arr.push(3);
            arr.pop();
            arr.set(1, 4);
            print arr.len();
            print arr.get(0);
            print arr.get(1);
        "#)?;
        MockLogger::entries(|entries| {
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].body, "2");
            assert_eq!(entries[1].body, "1");
            assert_eq!(entries[2].body, "4");
        });
        Ok(())
    }
}
