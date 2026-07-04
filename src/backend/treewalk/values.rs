use crate::types::token::Token;
use crate::types::ast::Stmt;
use super::TWInterp;
use crate::backend::treewalk::environment::Environment;

use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum Function {
    NativeFunction{arity: usize, imp: fn(&mut TWInterp, &[Value]) -> Result<Value, RuntimeError>},
    LoxFunction{
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    name: String
}

impl Class {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub fn to_string(&self) -> &str {
        &self.name
    }
}

// currently shadows literal, but with global strings
#[derive(Debug, Clone)]
pub enum Value {
    String(String), // heap strings for runtime?
    Number(f64),
    Boolean(bool),
    Nil,
    Function(Function),
    Class(Class)
}

pub enum Interrupt {
    Return{
        keyword: Token,
        value: Option<Value>
    },
    Error(RuntimeError)
}

impl From<RuntimeError> for Interrupt {
    fn from(e: RuntimeError) -> Self {
        Interrupt::Error(e)
    }
}

pub struct RuntimeError {
    pub message: String,
    pub line: usize
}

pub fn alv_stringify(value: &Value) -> String {
        match value {
            Value::String(s) => {
                s.clone()
            },
            Value::Boolean(b) => {
                b.to_string()
            },
            Value::Number(n) => {
                n.to_string()
            }
            Value::Nil => {
                "Nil".to_string()
            },
            Value::Function(f) => {
                match f {
                    Function::NativeFunction { .. } => { "<NATIVE FUNCTION>".to_string() },
                    Function::LoxFunction { name, .. } => { format!("<Fn {}>", name.lexeme) }
                }
            },
            Value::Class(c) => {
                c.to_string().to_string() // lol
            }
        }
    }