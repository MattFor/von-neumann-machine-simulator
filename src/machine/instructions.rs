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

pub struct OpcodeInfo {
    pub value: i32,
    pub opcode: Opcode,
    pub description: &'static str, // Change when implementing localization
}

pub const OPCODES: [OpcodeInfo; 12] = [
    OpcodeInfo {
        value: 0,
        opcode: Opcode::Nop,
        description: "No operation",
    },
    OpcodeInfo {
        value: 1,
        opcode: Opcode::Load,
        description: "Read the value stored at the memory address pointed to by the operand and copy it into the accumulator",
    },
    OpcodeInfo {
        value: 2,
        opcode: Opcode::Store,
        description: "Copy the current value from the accumulator into the memory address given by the operand",
    },
    OpcodeInfo {
        value: 3,
        opcode: Opcode::Add,
        description: "Add the operand value to the accumulator",
    },
    OpcodeInfo {
        value: 4,
        opcode: Opcode::Sub,
        description: "Subtract the operand value from the accumulator",
    },
    OpcodeInfo {
        value: 5,
        opcode: Opcode::Mul,
        description: "Multiply the accumulator by the operand value",
    },
    OpcodeInfo {
        value: 6,
        opcode: Opcode::Div,
        description: "Divide the accumulator by the operand value",
    },
    OpcodeInfo {
        value: 7,
        opcode: Opcode::Jump,
        description: "Set the program counter to the address given by the operand and continue execution from there",
    },
    OpcodeInfo {
        value: 8,
        opcode: Opcode::JumpIfZero,
        description: "If the accumulator is zero, set the program counter to the address given by the operand",
    },
    OpcodeInfo {
        value: 9,
        opcode: Opcode::Input,
        description: "Read input into the accumulator", // not sure
    },
    OpcodeInfo {
        value: 10,
        opcode: Opcode::Output,
        description: "Write the current value of the accumulator to the output",
    },
    OpcodeInfo {
        value: 255,
        opcode: Opcode::Halt,
        description: "Stop the program and halt execution",
    },
];

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: i32,
}
