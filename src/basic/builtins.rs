use super::value::*;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

lazy_static! {
    pub static ref BUILTINS: HashMap<String, LoxValue> = {
        let mut constants = HashMap::<String, LoxValue>::new();
        constants.insert(
            "time".into(),
            LoxFunction {
                name: Some("time".into()),
                params: vec![],
                body: FunctionBody::Native(|_| {
                    let now = SystemTime::now();
                    let elapsed = now.duration_since(UNIX_EPOCH)?;
                    Ok(LoxValue::Number(elapsed.as_millis() as f64))
                }),
                closure: None,
            }.into(),
        );
        constants
    };
}
