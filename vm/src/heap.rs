use core::fmt;

use crate::{chunk::Chunk, value::Value};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjRef(u32);

#[derive(PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Default)]
pub struct Function {
    pub arity:          u8,
    pub upvalue_count:  usize,  
    pub chunk:          Chunk,
    pub name:           Option<Box<str>>,
}

impl std::fmt::Display for ObjRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type NativeFn = fn(&[Value]) -> Value;

pub struct Closure {
    pub function:   ObjRef,
    pub upvalues:   Vec<ObjRef>,
}

pub enum Upvalue {
    Open(usize),    // on the stack
    Closed(Value),  // in the heap
}

pub enum Obj {
    String(Box<str>),
    Function(Function),
    Native(NativeFn),
    Closure(Closure),
    Upvalue(Upvalue),
}

#[derive(Default)]
pub struct Heap {
    objects: Vec<Obj>,
}

//
// IMPORTANT NOTE:
// we don't free anything (Sergey wants to skip gc)
// so for now we have a really shaky handle assignment.
// should be fine because eventually we will replace.
//

impl Heap {
    pub fn alloc(&mut self, obj: Obj) -> ObjRef {
        self.objects.push(obj);
        ObjRef(self.objects.len() as u32 - 1) // TODO: REALLY BAD NO WORKIE WHEN REMOVE STRINGS
    }

    pub fn alloc_string(&mut self, s: impl Into<Box<str>>) -> ObjRef {
        self.alloc(Obj::String(s.into()))
    }

    pub fn get(&self, r: ObjRef) -> &Obj {
        &self.objects[r.0 as usize]
    }

    pub fn get_mut(&mut self, r: ObjRef) -> &mut Obj {
        &mut self.objects[r.0 as usize]
    }

    pub fn get_upvalue(&self, r: ObjRef) -> &Upvalue {
        match self.get(r) {
            Obj::Upvalue(u) => u,
            _ => unreachable!(),
        }
    }

    pub fn get_upvalue_mut(&mut self, r: ObjRef) -> &mut Upvalue {
        match self.get_mut(r) {
            Obj::Upvalue(u) => u,
            _ => unreachable!(),
        }
    }

    pub fn get_str(&self, r: ObjRef) -> &str {
        match self.get(r) {
            Obj::String(s) => s,
            _ => unreachable!(), // only reachable if the compiler emitted bad bytecode
        }
    }

    pub fn get_fn(&self, r: ObjRef) -> &Function {
        match self.get(r) {
            Obj::Function(f) => f,
            _ => unreachable!(),
        }
    }

    pub fn get_closure(&self, r: ObjRef) -> &Closure {
        match self.get(r) {
            Obj::Closure(c) => c,
            _ => unreachable!(),
        }
    }
}
