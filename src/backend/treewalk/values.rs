use crate::types::token::Token;
use crate::types::ast::Stmt;
use super::TWInterp;
use crate::backend::treewalk::environment::Environment;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    pub name: String
}

impl Class {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    pub fn to_string(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    parent: Rc<Class>,
    fields: Rc<RefCell<HashMap<String, Value>>>
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self { parent: Rc::new(class), fields: Rc::new(RefCell::new(HashMap::new())) }
    }

    pub fn to_string(&self) -> String {
        format!("{} Instance", &self.parent.to_string())
    }

    // pub fn call(&self, interp: &TWInterp, args: Vec<Value>) -> Value {

    // }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            return Ok(self.fields.borrow().get(&name.lexeme).unwrap().clone());
        }
        Err(RuntimeError { message: format!("Undefined property '{}'.", &name.lexeme), line: name.line })
    }

    pub fn set(&self, name: &Token, value: &Value) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value.clone());
    }

    pub fn arity(&self) -> usize {
        0
    }

    pub fn equals(&self, other: &Instance) -> bool {
        Rc::ptr_eq(&self.fields, &other.fields)
    }

}

// currently shadows literal, but with global strings
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Function(Function),
    Class(Class),
    Instance(Instance)
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
                c.to_string()
            },
            Value::Instance(i) => {
                i.to_string()
            }
        }
    }