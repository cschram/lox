use super::{environment::LoxProperties, function::*, value::*};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn get_builtins() -> LoxProperties {
    let mut constants = LoxProperties::new();

    constants.insert(
        "time".into(),
        LoxFunction::native("time", vec![], |_| {
            let now = SystemTime::now();
            let elapsed = now.duration_since(UNIX_EPOCH)?;
            Ok(LoxValue::Number(elapsed.as_millis() as f64))
        })
        .into(),
    );

    constants.insert(
        "get_arg".into(),
        LoxFunction::native("get_arg", vec!["arg"], |args| {
            let arg = args[0].get_number()?;
            let args: Vec<String> = env::args().collect();
            Ok(args
                .get(arg as usize)
                .cloned()
                .map(LoxValue::String)
                .unwrap_or(LoxValue::Nil))
        })
        .into(),
    );

    constants
}
