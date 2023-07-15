use super::value::*;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn globals() -> HashMap<String, LoxValue> {
    let mut globals = HashMap::<String, LoxValue>::new();
    globals.insert(
        "time".into(),
        LoxValue::Fun {
            arity: 0,
            name: "<native fn time>".into(),
            fun: |_| {
                let now = SystemTime::now();
                let elapsed = now.duration_since(UNIX_EPOCH)?;
                Ok(LoxValue::Number(elapsed.as_millis() as f64))
            },
        },
    );
    globals
}
