#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Nop,
    Load,
    Store,
    Add,
    Sub,
    Mul,
    Div,

    Jump,
    JumpIfZero,

    Input,
    Output,

    Halt,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: i32,
}
