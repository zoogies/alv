use std::fmt;

use crate::heap::*;

#[derive(Clone, Copy)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(ObjRef)
}

pub struct ValueDisplay<'a> { value: Value, heap: &'a Heap }

impl Value {
    pub fn display<'a>(self, heap: &'a Heap) -> ValueDisplay<'a> {
        ValueDisplay{ value: self, heap }
    }
}

impl fmt::Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Value::Bool(b) => {
                if b { write!(f, "true") } else { write!(f, "false") }
            },
            Value::Nil => write!(f, "nil"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Obj(o) => {
                match self.heap.get(o) {
                    Obj::String(s) => write!(f, "{s}")
                }
            }
        }
    }
}

pub fn print_value(v: &Value, heap: &Heap) {
    print!("{}",v.display(heap))
}