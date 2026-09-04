use super::instructions::{Instruction, InstructionSet};

// 0x1234
// 0x12  - opcode
// 34    - operand
// >> 8  - bitshift by 8 bits
// 0x1234 -\> 0x0012
// after & 0xff it keeps the lowest 8 bits (0x12)
// (val >> 8) & 0xff extract 1 byte (opcode byte)
// second one extracts operand byte
// then both are converted into Instruction
pub fn decode(set: &InstructionSet, value: i32) -> Instruction {
    Instruction {
        opcode: set.opcode((value >> 8) & 0xff),
        operand: value & 0xff,
    }
}

pub fn encode(code: i32, operand: i32) -> i32 {
    ((code & 0xff) << 8) | (operand & 0xff)
}
