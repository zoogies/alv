use num_traits::FromPrimitive;

use crate::backend::vm::IError::RuntimeError;
use crate::common::*;
use crate::chunk::*;
use crate::debug::dissassemble_instruction;
use crate::value::*;
use crate::frontend::compiler::*;

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

    pub fn peek(&self, dist: usize) -> Value {
        self.stack[self.stack.len() - dist - 1] // TODO: validate if -1 is right here
    }

    pub fn reset(&mut self) {
        self.stack.clear();
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

fn is_falsey(v: Value) -> bool {
    match v {
        Value::Bool(b) => !b,
        Value::Nil => true,
        _ => false,
    }
}

fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    }
}

impl VM {
    // format all messages at call sites
    fn runtime_error(&mut self, msg: &str, chunk: &Chunk) {
        eprintln!("{msg}");
        eprintln!("[line {}] in script", chunk.lines[self.ip - 1]);
        self.stack.reset();
    }

    fn read_byte(&mut self, chunk: &Chunk) -> u8 {
        self.ip += 1;
        chunk.code[self.ip - 1]
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        chunk.constants[self.read_byte(chunk) as usize]
    }

    pub fn interpret(&mut self, input: &str) -> InterpretResult {
        let chunk = Compiler::compile(input)?;
        self.run(&chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        macro_rules! binary_op {
            ($ctor:path, $op:tt) => {{
                match (self.stack.peek(1), self.stack.peek(0)) {
                    (Value::Number(a), Value::Number(b)) => {
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push($ctor(a $op b));
                    }
                    _ => {
                        self.runtime_error("Operands must be numbers.", chunk);
                        return Err(RuntimeError);
                    }
                }
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
                OPCODE::Nil => self.stack.push(Value::Nil),
                OPCODE::True => self.stack.push(Value::Bool(true)),
                OPCODE::False => self.stack.push(Value::Bool(false)),
                OPCODE::Not => {
                    let &v = &self.stack.pop();
                    self.stack.push(Value::Bool(is_falsey(v)))
                },
                OPCODE::Add         =>  binary_op!(Value::Number, +),
                OPCODE::Subtract    =>  binary_op!(Value::Number, -),
                OPCODE::Multiply    =>  binary_op!(Value::Number, *),
                OPCODE::Divide      =>  binary_op!(Value::Number, /),
                OPCODE::Negate => {
                    let v = self.stack.pop();
                    
                    match v {
                        Value::Number(n) => {
                            self.stack.push(Value::Number(-n));
                        }
                        _ => {
                            self.runtime_error("Operand must be a number.", chunk);
                            return Err(RuntimeError);
                        }
                    }
                },
                OPCODE::Equal => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.stack.push(Value::Bool(values_equal(a, b)))
                },
                OPCODE::Greater => binary_op!(Value::Bool, >),
                OPCODE::Less => binary_op!(Value::Bool, <),
            }
        }
    }
} 