use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    #[default]
    Nop = 0,
    Load = 1,
    Store = 2,
    Add = 3,
    Sub = 4,
    Mul = 5,
    Div = 6,

    Jump = 7,
    JumpIfZero = 8,

    Input = 9,
    Output = 10,

    Inc = 11,
    Dec = 12,
    Null = 13,
    Tst = 14,

    Halt = 255,
}

// Workaround for iterating through enums
impl Opcode {
    const ALL: [Opcode; 16] = [
        Opcode::Nop,
        Opcode::Load,
        Opcode::Store,
        Opcode::Add,
        Opcode::Sub,
        Opcode::Mul,
        Opcode::Div,
        Opcode::Jump,
        Opcode::JumpIfZero,
        Opcode::Input,
        Opcode::Output,
        Opcode::Inc,
        Opcode::Dec,
        Opcode::Null,
        Opcode::Tst,
        Opcode::Halt,
    ];

    pub fn iter() -> impl Iterator<Item = Opcode> {
        Self::ALL.iter().copied()
    }

    pub const fn description(self) -> &'static str {
        match self {
            Opcode::Nop => "No operation",
            Opcode::Load => {
                "Read the value stored at the memory address pointed to by the operand and copy it into the accumulator"
            }
            Opcode::Store => {
                "Copy the current value from the accumulator into the memory address given by the operand"
            }
            Opcode::Add => {
                "Add the value stored at the memory address given by the operand to the accumulator"
            }
            Opcode::Sub => {
                "Subtract the value stored at the memory address given by the operand from the accumulator"
            }
            Opcode::Mul => "Multiply the accumulator by the operand value",
            Opcode::Div => "Divide the accumulator by the operand value",
            Opcode::Jump => {
                "Set the program counter to the address given by the operand and continue execution from there"
            }
            Opcode::JumpIfZero => {
                "If the accumulator is zero, set the program counter to the address given by the operand"
            }
            Opcode::Input => "Read a value from input into the accumulator",
            Opcode::Output => "Write the current value of the accumulator to the output",
            Opcode::Inc => {
                "Increment the value stored at the memory address given by the operand"
            }
            Opcode::Dec => {
                "Decrement the value stored at the memory address given by the operand"
            }
            Opcode::Null => {
                "Set the value stored at the memory address given by the operand to zero"
            }
            Opcode::Tst => {
                "If the value at the memory address given by the operand is zero, skip the next instruction"
            }
            Opcode::Halt => "Stop the program and halt execution",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Definition {
    pub code: i32,
    pub mnemonic: String,
    pub opcode: Opcode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSet {
    pub name: String,
    pub definitions: Vec<Definition>,
}

impl InstructionSet {
    pub fn find(&self, code: i32) -> Option<&Definition> {
        self.definitions
            .iter()
            .find(|definition| definition.code == code)
    }

    pub fn opcode(&self, code: i32) -> Opcode {
        self.find(code)
            .map_or(Opcode::Nop, |definition| definition.opcode)
    }

    pub fn mnemonic(&self, code: i32) -> String {
        self.find(code).map_or_else(
            || format!("{code:02X} - ?"),
            |definition| definition.mnemonic.clone(),
        )
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;

        Ok(())
    }
}

impl Default for InstructionSet {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),

            definitions: [
                (0, "NOP", Opcode::Nop),
                (1, "LOAD", Opcode::Load),
                (2, "STORE", Opcode::Store),
                (3, "ADD", Opcode::Add),
                (4, "SUB", Opcode::Sub),
                (5, "MUL", Opcode::Mul),
                (6, "DIV", Opcode::Div),
                (7, "JMP", Opcode::Jump),
                (8, "JZ", Opcode::JumpIfZero),
                (9, "IN", Opcode::Input),
                (10, "OUT", Opcode::Output),
                (11, "INC", Opcode::Inc),
                (12, "DEC", Opcode::Dec),
                (13, "NULL", Opcode::Null),
                (14, "TST", Opcode::Tst),
                (255, "HALT", Opcode::Halt),
            ]
            .into_iter()
            .map(|(code, mnemonic, opcode)| Definition {
                code,
                mnemonic: mnemonic.to_string(),
                opcode,
            })
            .collect(),
        }
    }
}
