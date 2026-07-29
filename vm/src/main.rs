use alv_vm::chunk::*;
use alv_vm::debug::*;

fn main() {
    let mut c = Chunk::default();
    c.write(OPCODE::Return as u8);
    dissassemble_chunk(&c, "test");
}
