#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Nop         = 0,
    Load        = 1,
    Store       = 2,
    Add         = 3,
    Sub         = 4,
    Mul         = 5,
    Div         = 6,

    Jump        = 7,
    JumpIfZero  = 8,

    Input       = 9,
    Output      = 10,

    Halt        = 255,
}

// Workaround for iterating through enums
impl Opcode {
    const ALL: [Opcode; 12] = [Opcode::Nop, Opcode::Load, Opcode::Store, Opcode::Add,
                                Opcode::Sub, Opcode::Mul, Opcode::Div, Opcode::Jump,
                                Opcode::JumpIfZero, Opcode::Input, Opcode::Output, Opcode::Halt,];

    pub fn iter() -> impl Iterator<Item = Opcode> {
        Self::ALL.iter().copied()
    }
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
