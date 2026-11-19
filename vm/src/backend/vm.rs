use num_traits::FromPrimitive;
use rustc_hash::FxHashMap;

use crate::backend::vm::IError::RuntimeError;
use crate::common::*;
use crate::chunk::*;
use crate::debug::dissassemble_instruction;
use crate::natives::clock;
use crate::value::*;
use crate::frontend::compiler::*;
use crate::heap::*;

const STACK_MAX: usize = FRAMES_MAX * (u8::MAX as usize + 1);

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

    pub fn get(&self, slot: usize) -> Value {
        self.stack[slot]
    }

    pub fn set(&mut self, slot: usize, value: Value) {
        self.stack[slot] = value
    }
}

struct CallFrame {
    closure:    ObjRef,
    ip:         usize,
    slots:      usize,
}

#[derive(Default)]
pub struct VM {
    frames:     Vec<CallFrame>,
    stack:      Stack,
    heap:       Heap,
    globals:    FxHashMap<Box<str>, Value>,
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

fn values_equal(a: Value, b: Value, heap: &Heap) -> bool {
    match (a, b) {
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        (Value::Obj(a), Value::Obj(b)) => {
            match (heap.get(a), heap.get(b)) {
                (Obj::String(a), Obj::String(b)) => a == b,
                _ => a == b,
            }
        }
        _ => false,
    }
}

impl VM {

    pub fn define_native(&mut self, name: &str, f : NativeFn) {
        let r = self.heap.alloc(Obj::Native(f));
        self.globals.insert(name.into(), Value::Obj(r));
    }

    fn register_stdlib(&mut self) {
        self.define_native("clock", clock);
    }

    fn frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("Zero frames in existence during call to frame()")
    }

    fn chunk(&self) -> &Chunk {
        let f = &self.frames.last().expect("Tried to get chunk while there are zero frames.").closure;
        let c = self.heap.get_closure(*f);
        &self.heap.get_fn(c.function).chunk
    }

    // format all messages at call sites
    fn runtime_error(&mut self, msg: &str) {
        eprintln!("{msg}");

        for f in self.frames.iter().rev() {
            let closure = self.heap.get_closure(f.closure);
            let function = self.heap.get_fn(closure.function);
            let line = function.chunk.lines[f.ip - 1];
            let place = match function.name.as_deref() {
                Some(n) => format!("{n}()"),
                None => "script".to_string(),
            };
            eprintln!("[line {}] in {}", line, place);
        }

        self.stack.reset();
        self.frames.clear();
        self.stack.reset();
    }

    fn read_byte(&mut self) -> u8 {
        self.frame().ip += 1;
        let ip = self.frame().ip;
        self.chunk().code[ip - 1]
    }

    fn read_constant(&mut self) -> Value {
        let b = self.read_byte();
        self.chunk().constants[b as usize]
    }

    fn read_short(&mut self) -> u16 {
        let b1 = self.read_byte();
        let b2 = self.read_byte();
        u16::from_be_bytes([b1, b2])
    }

    fn read_string(&mut self) -> ObjRef {
        let b = self.read_byte();
        match self.chunk().constants[b as usize] {
            Value::Obj(o) => o,
            _ => panic!("wtf")
        }
    }

    fn call(&mut self, closure: ObjRef, arg_count: u8) -> bool {
        match self.heap.get(closure) {
            Obj::Closure(c) => {
                let f = self.heap.get_fn(c.function);
                if arg_count != f.arity {
                    self.runtime_error(&format!("Expected {} arguments but got {}.", f.arity, arg_count));
                    return false
                }
            },
            _ => unreachable!()
        }

        if self.frames.len() == FRAMES_MAX {
            self.runtime_error("Stack overflow.");
            return false
        }

        self.frames.push(CallFrame {
            closure,
            ip: 0,
            slots: self.stack.top() - arg_count as usize - 1,
        });
        true
    }

    fn call_value(&mut self, callee: &Value, arg_count: u8) -> bool {
        if let Value::Obj(r) = callee {
            match self.heap.get(*r) {
                Obj::Closure(_) => return self.call(*r, arg_count),
                Obj::Native(nf) => {
                    let values = &self.stack.stack.split_off(self.stack.stack.len() - arg_count as usize);
                    let rv = nf(values);
                    self.stack.pop();
                    self.stack.push(rv);
                    return true;
                }
                _ => ()
            }
        }
        self.runtime_error("Can only call functions and classes.");
        false
    }

    pub fn interpret(&mut self, input: &str) -> InterpretResult {
        let f = Compiler::compile(input, &mut self.heap)?;
        let c = self.heap.alloc(Obj::Closure(Closure {
            function: f,
            upvalues: vec![],
        }));
        self.stack.push(Value::Obj(c));
        self.call(c, 0);
        
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        macro_rules! binary_op {
            ($ctor:path, $op:tt) => {{
                match (self.stack.peek(1), self.stack.peek(0)) {
                    (Value::Number(a), Value::Number(b)) => {
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push($ctor(a $op b));
                    }
                    _ => {
                        self.runtime_error("Operands must be numbers.");
                        return Err(RuntimeError);
                    }
                }
            }};
        }
        
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("        ");
                for v in &self.stack.stack {
                    print!("[ ");
                    print_value(v, &self.heap);
                    print!(" ]");
                }
                println!();
                let ip = self.frame().ip;
                let c = self.chunk();
                dissassemble_instruction(c, ip, &self.heap);
            }

            let Some(op) = OPCODE::from_u8(self.read_byte()) else { panic!("Fuck you, Rust compiler!") };
            match op {
                OPCODE::Closure => {
                    let c = match self.read_constant() {
                        Value::Obj(r) => r,
                        _ => unreachable!()
                    };
                    let v = self.heap.alloc(Obj::Closure(Closure {
                        function: c,
                        upvalues: vec![],
                    }));
                    self.stack.push(Value::Obj(v));
                },
                OPCODE::Call => {
                    let arg_count = self.read_byte();
                    let p = self.stack.peek(arg_count as usize);
                    if !self.call_value(&p, arg_count) {
                        return Err(RuntimeError)
                    }
                },
                OPCODE::Loop => {
                    let offset = self.read_short();
                    self.frame().ip -= offset as usize;
                },
                OPCODE::Jump => {
                    let offset = self.read_short() as usize;
                    self.frame().ip += offset;
                },
                OPCODE::JumpIfFalse => {
                    let offset = self.read_short() as usize;
                    if is_falsey(self.stack.peek(0)) {
                        self.frame().ip += offset;
                    }
                },
                OPCODE::GetLocal => {
                    let slot = self.read_byte();
                    let base = self.frame().slots;
                    self.stack.push(self.stack.get(base + slot as usize));
                },
                OPCODE::SetLocal => {
                    let slot = self.read_byte();
                    let base = self.frame().slots;
                    self.stack.set(base + slot as usize, self.stack.peek(0))
                },
                OPCODE::SetGlobal => {
                    let h = self.read_string();
                    let r = self.heap.get_str(h);

                    let Some(g) = self.globals.get_mut(r) else {
                        self.runtime_error(&format!("Undefined variable '{}'.", r));
                        return Err(RuntimeError);
                    };

                    *g = self.stack.peek(0);
                },
                OPCODE::DefineGlobal => {
                    let h = self.read_string();
                    let r = self.heap.get_str(h);
                    self.globals.insert(r.into(), self.stack.peek(0));
                    self.stack.pop();
                },
                OPCODE::GetGlobal => {
                    let h = self.read_string();
                    let r = self.heap.get_str(h);
                    let Some(g) = self.globals.get(r) else {
                        self.runtime_error(&format!("Undefined variable '{}'.", r));
                        return Err(RuntimeError);
                    };
                    self.stack.push(*g);
                },
                OPCODE::Pop => {
                    self.stack.pop();
                }
                OPCODE::Print => {
                    print_value(&self.stack.pop(), &self.heap);
                    println!();
                },
                OPCODE::Return => {
                    let v = self.stack.pop();
                    let returning = self.frames.pop().expect("Somehow tried to frame pop with zero frames.");
                    if self.frames.is_empty() {
                        self.stack.pop();
                        return Ok(())
                    }

                    self.stack.stack.truncate(returning.slots);
                    self.stack.push(v);
                },
                OPCODE::Constant => {
                    let constant: Value = self.read_constant();
                    self.stack.push(constant);
                },
                OPCODE::Nil => self.stack.push(Value::Nil),
                OPCODE::True => self.stack.push(Value::Bool(true)),
                OPCODE::False => self.stack.push(Value::Bool(false)),
                OPCODE::Not => {
                    let &v = &self.stack.pop();
                    self.stack.push(Value::Bool(is_falsey(v)))
                },
                OPCODE::Add => {
                    match (self.stack.peek(0), self.stack.peek(1)) {
                        (Value::Obj(a), Value::Obj(b)) => {
                            match (self.heap.get(a), self.heap.get(b)) {
                                (Obj::String(a), Obj::String(b)) => {
                                    let r = self.heap.alloc_string(format!("{b}{a}"));
                                    
                                    self.stack.pop();
                                    self.stack.pop();
                                    self.stack.push(Value::Obj(r));
                                },
                                _ => {
                                    self.runtime_error("Operands must be two numbers or two strings.");
                                    return Err(RuntimeError);
                                }
                            }
                        },
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.pop();
                            self.stack.pop();
                            self.stack.push(Value::Number(a + b));
                        }
                        _ => {
                            self.runtime_error("Operands must be two numbers or two strings.");
                            return Err(RuntimeError);
                        }
                    }
                } 
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
                            self.runtime_error("Operand must be a number.");
                            return Err(RuntimeError);
                        }
                    }
                },
                OPCODE::Equal => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.stack.push(Value::Bool(values_equal(a, b, &self.heap)))
                },
                OPCODE::Greater => binary_op!(Value::Bool, >),
                OPCODE::Less => binary_op!(Value::Bool, <),
            }
        }
    }

    pub fn new() -> Self {
        let mut vm = VM::default();

        if !ALV_NOSTD {
            vm.register_stdlib();
        }

        vm
    }

}