use std::time::{SystemTime, UNIX_EPOCH};

use super::environment::*;
use super::values::*;
use super::*;

pub fn register_natives(env: &mut Environment) {
    env.define("clock".to_string(), Value::Function(Function::NativeFunction { arity: 0, imp: clock }));
}

fn clock(_interp: &mut TWInterp, _args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(
        Value::Number(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64() as f64)
    )
}