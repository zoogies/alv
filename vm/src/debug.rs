use crate::chunk::*;
use num_traits::FromPrimitive;
pub fn dissassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = dissassemble_instruction(chunk, offset);
    }
}

pub fn dissassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    let Some(instr) = OPCODE::from_u8(chunk.code[offset]) else { panic!("Fuck you, Rust compiler!") };
    match instr {
        OPCODE::Return => {
            simple_instruction("OP_RETURN", offset)
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}