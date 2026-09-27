use std::fmt;

#[derive(Clone, Copy)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => {
                if *b { write!(f, "true") } else { write!(f, "false") }
            },
            Value::Nil => write!(f, "nil"),
            Value::Number(v) => write!(f, "{v}"),
        }
    }
}

pub fn print_value(v: &Value) {
    print!("{}",*v)
}