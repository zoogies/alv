use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::values::*;

#[derive(Default, Debug)]
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

    pub fn get_at(&self, distance: usize, name: &str) -> Option<Value> {
        if distance == 0 {
            return self.environment.get(name).cloned();
        }

        let mut env = Rc::clone(self.enclosing.as_ref()?);
        for _ in 1..distance {
            let next = Rc::clone(env.borrow().enclosing.as_ref()?);
            env = next;
        }

        env.borrow().environment.get(name).cloned()
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

    pub fn assign_at(&mut self, dist: usize, k: &str, v: Value) {
        if dist == 0 {
            self.environment.insert(k.to_string(), v);
            return;
        }

        let mut env = Rc::clone(self.enclosing.as_ref().unwrap());
        for _ in 1..dist {
            let next = Rc::clone(env.borrow().enclosing.as_ref().unwrap());
            env = next;
        }
        env.borrow_mut().environment.insert(k.to_string(), v);
    }
}