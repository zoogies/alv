#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjRef(u32);

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

    pub fn get(&self, r: ObjRef) -> &Obj {
        &self.objects[r.0 as usize]
    }
}
