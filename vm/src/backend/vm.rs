use num_traits::FromPrimitive;

use crate::chunk::*;
use crate::debug::dissassemble_instruction;
use crate::value::*;

#[derive(Default)]
pub struct VM {
    ip: usize,
}

pub enum IError {
    CompileError,
    RuntimeError,
}

pub type InterpretResult = Result<(), IError>;

// I don't want to have to mentally think about cargo feature flags.
const DEBUG_TRACE_EXECUTION: bool = true;

impl VM {
    fn read_byte(&mut self, chunk: &Chunk) -> u8 {
        self.ip += 1;
        chunk.code[self.ip - 1]
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        chunk.constants[self.read_byte(chunk) as usize]
    }

    pub fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        
        loop {
            if DEBUG_TRACE_EXECUTION {
                dissassemble_instruction(chunk, self.ip);
            }

            let Some(op) = OPCODE::from_u8(self.read_byte(chunk)) else { panic!("Fuck you, Rust compiler!") };
            match op {
                OPCODE::Return => {
                    return Ok(());
                },
                OPCODE::Constant => {
                    let constant: Value = self.read_constant(chunk);
                    print_value(&constant);
                    println!();
                    continue;
                },
            }
        }
    }
} 