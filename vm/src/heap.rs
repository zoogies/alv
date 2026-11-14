use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjRef(u32);

impl std::fmt::Display for ObjRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum Obj {
    String(Box<str>),
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

    pub fn get_str(&self, r: ObjRef) -> &str {
        match self.get(r) {
            Obj::String(s) => s
        }
    }
}
