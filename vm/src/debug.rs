use num_traits::FromPrimitive;

use crate::chunk::*;
use crate::value::*;

pub fn dissassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = dissassemble_instruction(chunk, offset);
    }
}

pub fn dissassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    // lineno
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    }
    else {
        print!("{:>4} ", chunk.lines[offset])
    }

    let Some(instr) = OPCODE::from_u8(chunk.code[offset]) else { panic!("Fuck you, Rust compiler!") };
    match instr {
        OPCODE::Return => {
            simple_instruction("OP_RETURN", offset)
        },
        OPCODE::Constant => {
            constant_instruction("OP_CONSTANT", chunk, offset)
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1];
    print!("{:<16} {:>4} '", name, constant);
    print_value(&chunk.constants[constant as usize]);
    print!("'\n");
    offset + 2
}