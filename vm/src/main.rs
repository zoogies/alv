use alv_vm::chunk::*;
use alv_vm::debug::*;
use alv_vm::value::*;

// TODO: some template T to avoid "as u8"?
fn main() {
    let mut c = Chunk::default();
    c.write_code(OPCODE::Return as u8, 123);

    let cost = c.add_constant(0.000001 as Value);
    c.write_code(OPCODE::Constant as u8, 123);
    c.write_code(cost as u8, 123);

    dissassemble_chunk(&c, "test");
}
