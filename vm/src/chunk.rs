use num_derive::FromPrimitive;    

#[repr(u8)]
#[derive(FromPrimitive)]
pub enum OPCODE {
    Return
}

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<u8>
}

impl Chunk {
    pub fn write(&mut self, v: u8) {
        self.code.push(v);
    }
}