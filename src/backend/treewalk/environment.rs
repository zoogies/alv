use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;

use super::values::*;

#[derive(Default, Debug)]
pub struct Environment {
    pub environment: rustc_hash::FxHashMap<String,Value>,
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
        if let Some(slot) = self.environment.get_mut(k) {
            *slot = v;
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
            if let Some(slot) = self.environment.get_mut(k) { *slot = v; }
            return;
        }

        let mut env = Rc::clone(self.enclosing.as_ref().unwrap());
        for _ in 1..dist {
            let next = Rc::clone(env.borrow().enclosing.as_ref().unwrap());
            env = next;
        }
        if let Some(slot) = env.borrow_mut().environment.get_mut(k) { *slot = v; }
    }

    pub fn from_enclosing(enclosing: &Rc<RefCell<Self>>) -> Self {
        Self {
            environment: FxHashMap::default(),
            enclosing: Some(Rc::clone(enclosing))
        }
    }
}