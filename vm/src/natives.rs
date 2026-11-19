use std::time::{SystemTime, UNIX_EPOCH};

use crate::value::Value;

pub fn clock(_args: &[Value]) -> Value {
    Value::Number(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs_f64())
}