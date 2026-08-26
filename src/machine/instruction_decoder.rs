use super::instructions::{Instruction, OPCODES, Opcode};

pub fn decode(value: i32) -> Instruction {
    let code = (value >> 8) & 0xff;
    let operand = value & 0xff;

    let opcode = OPCODES
        .iter()
        .find(|candidate| candidate.value == code)
        .map_or(Opcode::Nop, |candidate| candidate.opcode);

    Instruction { opcode, operand }
}

pub fn encode(instruction: Instruction) -> i32 {
    ((instruction.opcode as i32) << 8) | (instruction.operand & 0xff)
}
