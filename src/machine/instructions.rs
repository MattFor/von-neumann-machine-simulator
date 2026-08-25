#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    Halt = 255,
}

pub const OPCODES: [(i32, Opcode); 12] = [
    (0, Opcode::Nop),
    (1, Opcode::Load),
    (2, Opcode::Store),
    (3, Opcode::Add),
    (4, Opcode::Sub),
    (5, Opcode::Mul),
    (6, Opcode::Div),
    (7, Opcode::Jump),
    (8, Opcode::JumpIfZero),
    (9, Opcode::Input),
    (10, Opcode::Output),
    (255, Opcode::Halt),
];

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: i32,
}
