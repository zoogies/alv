use crate::{backend::treewalk::values::Function::LoxFunction, types::token::Token};
use crate::types::ast::FuncDecl;
use super::TWInterp;
use crate::backend::treewalk::environment::Environment;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub enum Function {
    NativeFunction{arity: usize, imp: fn(&mut TWInterp, &[Value]) -> Result<Value, RuntimeError>},
    LoxFunction{
        decl: Rc<FuncDecl>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    }
}

impl Function {
    pub fn bind(&self, inst: &Instance, line: usize) -> Result<Self, RuntimeError> {
        match self {
            LoxFunction { decl, is_initializer, closure } => {
                let e = Rc::new(RefCell::new(Environment::from_enclosing(closure)));

                e.borrow_mut().define("this".to_string(), Value::Instance(inst.clone()));

                Ok(Function::LoxFunction { decl: Rc::clone(decl), closure: e, is_initializer: *is_initializer })
            },
            _ => {
                Err(RuntimeError { message: format!("Tried binding to non-lox function!"), line: line })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Function>,
    pub superclass: Option<Rc<Class>>,
}

impl Class {
    pub fn new(name: &str, methods: HashMap<String, Function>, superclass: Option<Rc<Class>>) -> Self {
        Self { name: name.to_string(), methods, superclass }
    }

    pub fn to_string(&self) -> String {
        self.name.clone()
    }

    pub fn find_method(&self, name: &str) -> Option<&Function> {
        if self.methods.contains_key(name) {
            return self.methods.get(name);
        }
        if let Some(sc) = &self.superclass {
            return sc.find_method(name);
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub parent: Rc<Class>,
    fields: Rc<RefCell<HashMap<String, Value>>>
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self { parent: class, fields: Rc::new(RefCell::new(HashMap::new())) }
    }

    pub fn to_string(&self) -> String {
        format!("{} Instance", &self.parent.to_string())
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(v) = self.fields.borrow().get(&name.lexeme) {
            return Ok(v.clone());
        }

        if let Some(meth) = self.parent.find_method(&name.lexeme) {
            return Ok(Value::Function(meth.clone().bind(&self.clone(), name.line)?))
        }

        Err(RuntimeError { message: format!("Undefined property '{}'.", &name.lexeme), line: name.line })
    }

    pub fn set(&self, name: &Token, value: &Value) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value.clone());
    }

    pub fn arity(&self) -> usize {
        if let Some(initializer) = self.parent.find_method("init") {
            match initializer {
                Function::NativeFunction { arity, .. } => { *arity },
                Function::LoxFunction { decl, .. } => { decl.params.len() }
            }
        } else {
            0
        }
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
    Class(Rc<Class>),
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
                    Function::LoxFunction { decl, .. } => { format!("<Fn {}>", decl.name.lexeme) }
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