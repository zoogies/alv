use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::values::*;

#[derive(Default)]
pub struct Environment {
    pub environment: HashMap<String,Value>,
    pub enclosing: Option<Rc<RefCell<Environment>>>
}

impl Environment {
    pub fn get(&self, k: &str) -> Option<Value> {
        match self.environment.get(k) {
            Some(v) => Some(v.clone()),
            None => match &self.enclosing {
                Some(parent) => parent.borrow().get(k),
                None => None
            }
        }
    }

    pub fn define(&mut self, k: String, v: Value) {
        self.environment.insert(k, v);
    }

    pub fn assign(&mut self, k: &str, v: Value) -> bool {
        if self.environment.contains_key(k) {
            self.environment.insert(k.to_string(), v);
            true
        }
        else {
            match &self.enclosing {
                Some(parent) => parent.borrow_mut().assign(k, v),
                None => false,
            }
        }
    }
}