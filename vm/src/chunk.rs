use num_derive::FromPrimitive;    

use crate::value::Value;

#[repr(u8)]
#[derive(FromPrimitive)]
pub enum OPCODE {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl From<OPCODE> for u8 {
    fn from(op: OPCODE) -> Self {
        op as u8
    }
}


#[derive(Default)]
pub struct Chunk {
    pub code:       Vec<u8>,
    pub constants:  Vec<Value>,
    pub lines:      Vec<u32>,
    // ^ TODO: this would be a good optimization when we diverge from CI,
    //         keeping this around is memory intensive
}

impl Chunk {
    pub fn write_code<T: Into<u8>>(&mut self, v: T, line: u32) {
        self.code.push(v.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, v: Value) -> usize {
        self.constants.push(v);
        self.constants.len() - 1
    }
}