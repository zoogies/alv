use num_traits::FromPrimitive;

use crate::chunk::*;
use crate::value::*;
use crate::heap::*;

pub fn dissassemble_chunk(chunk: &Chunk, name: &str, heap: &Heap) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = dissassemble_instruction(chunk, offset, heap);
    }
}

pub fn dissassemble_instruction(chunk: &Chunk, offset: usize, heap: &Heap) -> usize {
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
        OPCODE::Return =>       simple_instruction("OP_RETURN", offset),
        OPCODE::Constant =>     constant_instruction("OP_CONSTANT", chunk, offset, heap),
        OPCODE::Negate =>       simple_instruction("OP_NEGATE", offset),
        OPCODE::Add =>          simple_instruction("OP_ADD", offset),
        OPCODE::Subtract =>     simple_instruction("OP_SUBTRACT", offset),
        OPCODE::Multiply =>     simple_instruction("OP_MULTIPLY", offset),
        OPCODE::Divide =>       simple_instruction("OP_DIVIDE", offset),
        OPCODE::True =>         simple_instruction("OP_TRUE", offset),
        OPCODE::False =>        simple_instruction("OP_FALSE", offset),
        OPCODE::Nil =>          simple_instruction("OP_NIL", offset),
        OPCODE::Not =>          simple_instruction("OP_NOT", offset),
        OPCODE::Equal =>        simple_instruction("OP_EQUAL", offset),
        OPCODE::Greater =>      simple_instruction("OP_GREATER", offset),
        OPCODE::Less =>         simple_instruction("OP_LESS", offset),
        OPCODE::Print =>        simple_instruction("OP_PRINT", offset),
        OPCODE::Pop =>          simple_instruction("OP_POP", offset),
        OPCODE::DefineGlobal => constant_instruction("OP_DEFINE_GLOBAL", chunk, offset, heap),
        OPCODE::GetGlobal =>    constant_instruction("OP_GET_GLOBAL", chunk, offset, heap),
        OPCODE::SetGlobal =>    constant_instruction("OP_SET_GLOBAL", chunk, offset, heap),
        OPCODE::GetLocal =>     byte_instruction("OP_GET_LOCAL", chunk, offset), 
        OPCODE::SetLocal =>     byte_instruction("OP_SET_LOCAL", chunk, offset), 
        OPCODE::Jump =>         jump_instruction("OP_JUMP", 1, chunk, offset),
        OPCODE::JumpIfFalse =>  jump_instruction("OP_JUMP_IF_FALSE", 1, chunk, offset),
        OPCODE::Loop =>         jump_instruction("OP_LOOP", -1, chunk, offset),
        OPCODE::Call =>         byte_instruction("OP_CALL", chunk, offset),
        OPCODE::Closure => {
            let constant = chunk.code[offset + 1];
            print!("{:<16} {:>4} ", "OP_CLOSURE", constant);
            print_value(&chunk.constants[constant as usize], heap);
            print!("\n");

            let Value::Obj(r) = chunk.constants[constant as usize] else { unreachable!() };
            let function = heap.get_fn(r);

            let mut offset = offset + 2;
            for _ in 0..function.upvalue_count {
                let is_local = chunk.code[offset];
                let index = chunk.code[offset + 1];
                let kind = if is_local == 1 { "local" } else { "upvalue" };
                println!("{:04}    |                     {} {}", offset, kind, index);
                offset += 2
            }
            offset
        },
        OPCODE::GetUpvalue =>   byte_instruction("OP_GET_UPVALUE", chunk, offset),
        OPCODE::SetUpvalue =>   byte_instruction("OP_SET_UPVALUE", chunk, offset),
        OPCODE::CloseUpvalue => simple_instruction("OP_CLOSE_UPVALUE", offset),
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize, heap: &Heap) -> usize {
    let constant = chunk.code[offset + 1];
    print!("{:<16} {:>4} '", name, constant);
    print_value(&chunk.constants[constant as usize], heap);
    print!("'\n");
    offset + 2
}

fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.code[offset + 1];
    println!("{:<16} {:>4}", name, slot);
    offset + 2
}

fn jump_instruction(name: &str, sign: isize, chunk: &Chunk, offset: usize) -> usize {
    let jump= u16::from_be_bytes([chunk.code[offset + 1], chunk.code[offset + 2]]) as usize;
    println!("{:<16} {:>4} -> {}", name, offset, (offset as isize) + 3 + sign * (jump as isize));
    offset + 3
}