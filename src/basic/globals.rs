use super::{value::*, ast::*};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn globals() -> HashMap<String, LoxValue> {
    let mut globals = HashMap::<String, LoxValue>::new();
    globals.insert(
        "time".into(),
        LoxValue::Function {
            name: Some("time".into()),
            params: vec![],
            body: FunctionBody::Native(|_, _| {
                    let now = SystemTime::now();
                    let elapsed = now.duration_since(UNIX_EPOCH)?;
                    Ok(LoxValue::Number(elapsed.as_millis() as f64))
                }
            ),
        },
    );
    globals
}
