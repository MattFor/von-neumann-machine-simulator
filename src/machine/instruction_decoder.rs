use super::instructions::{Instruction, Opcode};

pub fn decode(value: i32) -> Instruction {
    let opcode = (value >> 8) & 0xff;
    let operand = value & 0xff;

    let opcode = match opcode {
        0 => Opcode::Nop,
        1 => Opcode::Load,
        2 => Opcode::Store,
        3 => Opcode::Add,
        4 => Opcode::Sub,
        5 => Opcode::Mul,
        6 => Opcode::Div,
        7 => Opcode::Jump,
        8 => Opcode::JumpIfZero,
        9 => Opcode::Input,
        10 => Opcode::Output,
        255 => Opcode::Halt,
        _ => Opcode::Nop,
    };

    Instruction { opcode, operand }
}
