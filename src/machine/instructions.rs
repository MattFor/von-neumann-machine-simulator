#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: i32,
}
