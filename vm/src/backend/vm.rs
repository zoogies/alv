use num_traits::FromPrimitive;

use crate::chunk::*;
use crate::debug::dissassemble_instruction;
use crate::value::*;
use crate::compiler::*;

const STACK_MAX: usize = 256;

#[derive(Default)]
struct Stack {
    stack: Vec<Value>,
}

impl Stack {
    pub fn top(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, v: Value) {
        if self.stack.len() == STACK_MAX {
            panic!("Stack overflow!")
        }
        self.stack.push(v);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().expect("Tried to pop from empty stack!")
    }
}

#[derive(Default)]
pub struct VM {
    ip:     usize,
    stack:  Stack,
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

    pub fn interpret(&mut self, input: &str) -> InterpretResult {
        let comp = Compiler::default();

        Ok(())
    }

    pub fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        macro_rules! binary_op {
            ($op:tt) => {{
                let b = self.stack.pop();
                let a = self.stack.pop();
                self.stack.push(a $op b);
            }};
        }
        
        self.ip = 0;
        
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("        ");
                for v in &self.stack.stack {
                    print!("[ ");
                    print_value(v);
                    print!(" ]");
                }
                println!();
                dissassemble_instruction(chunk, self.ip);
            }

            let Some(op) = OPCODE::from_u8(self.read_byte(chunk)) else { panic!("Fuck you, Rust compiler!") };
            match op {
                OPCODE::Return => {
                    print_value(&self.stack.pop());
                    println!();
                    return Ok(());
                },
                OPCODE::Constant => {
                    let constant: Value = self.read_constant(chunk);
                    self.stack.push(constant);
                },
                OPCODE::Add         =>  binary_op!(+),
                OPCODE::Subtract    =>  binary_op!(-),
                OPCODE::Multiply    =>  binary_op!(*),
                OPCODE::Divide      =>  binary_op!(/),
                OPCODE::Negate => {
                    let v = -self.stack.pop();
                    self.stack.push(v);
                },
            }
        }
    }
} 