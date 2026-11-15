use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Format, UnknownCode};
use crate::utilities::read_text;

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

    // Runs the micro steps stored in its definition.
    Custom = 256,
}

// One elementary move inside the machine; every instruction is a list of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MicroOp {
    DbToRam,
    RamToDb,
    DbToIns,
    InsToAb,
    InsToDb,
    InsToPc,
    InsToPcIfZero,
    InsToPcIfNonNegative,
    InsToMc,
    McZero,
    PcToAb,
    PcToDb,
    PcInc,
    PcIncIfZero,
    PcIncIfPositive,
    PcIncIfNegative,
    DbToAb,
    AbToPc,
    AccZero,
    AccInc,
    AccDec,
    Plus,
    Minus,
    Times,
    Divide,
    AccToDb,
    DbToAcc,
    Input,
    Output,
    Stop,
}

// What the control unit does before every instruction.
pub const FETCH: [MicroOp; 4] = [
    MicroOp::PcToAb,
    MicroOp::RamToDb,
    MicroOp::DbToIns,
    MicroOp::InsToMc,
];

impl MicroOp {
    pub const ALL: [MicroOp; 30] = [
        MicroOp::DbToRam,
        MicroOp::RamToDb,
        MicroOp::DbToIns,
        MicroOp::InsToAb,
        MicroOp::InsToDb,
        MicroOp::InsToPc,
        MicroOp::InsToPcIfZero,
        MicroOp::InsToPcIfNonNegative,
        MicroOp::InsToMc,
        MicroOp::McZero,
        MicroOp::PcToAb,
        MicroOp::PcToDb,
        MicroOp::PcInc,
        MicroOp::PcIncIfZero,
        MicroOp::PcIncIfPositive,
        MicroOp::PcIncIfNegative,
        MicroOp::DbToAb,
        MicroOp::AbToPc,
        MicroOp::AccZero,
        MicroOp::AccInc,
        MicroOp::AccDec,
        MicroOp::Plus,
        MicroOp::Minus,
        MicroOp::Times,
        MicroOp::Divide,
        MicroOp::AccToDb,
        MicroOp::DbToAcc,
        MicroOp::Input,
        MicroOp::Output,
        MicroOp::Stop,
    ];

    // The number a micro step has in a Johnny microcode file.
    fn from_johnny(id: u32) -> Option<MicroOp> {
        Some(match id {
            1 => MicroOp::DbToRam,
            2 => MicroOp::RamToDb,
            3 => MicroOp::DbToIns,
            4 => MicroOp::InsToAb,
            5 => MicroOp::InsToMc,
            7 => MicroOp::McZero,
            8 => MicroOp::PcToAb,
            9 => MicroOp::PcInc,
            10 => MicroOp::PcIncIfZero,
            11 => MicroOp::InsToPc,
            12 => MicroOp::AccZero,
            13 => MicroOp::Plus,
            14 => MicroOp::Minus,
            15 => MicroOp::AccToDb,
            16 => MicroOp::AccInc,
            17 => MicroOp::AccDec,
            18 => MicroOp::DbToAcc,
            19 => MicroOp::Stop,
            _ => return None,
        })
    }

    // Button label, then what it does.
    pub const fn info(self) -> (&'static str, &'static str) {
        match self {
            MicroOp::DbToRam => (
                "db->ram",
                "Write the data bus into the memory cell the address bus points at",
            ),
            MicroOp::RamToDb => (
                "ram->db",
                "Read the memory cell the address bus points at onto the data bus",
            ),
            MicroOp::DbToIns => ("db->ins", "Copy the data bus into the instruction register"),
            MicroOp::InsToAb => (
                "ins->ab",
                "Put the operand of the instruction register on the address bus",
            ),
            MicroOp::InsToDb => (
                "ins->db",
                "Put the operand of the instruction register on the data bus",
            ),
            MicroOp::InsToPc => (
                "ins->pc",
                "Jump: copy the operand of the instruction register into the program counter",
            ),
            MicroOp::InsToPcIfZero => ("=0:ins->pc", "Jump only if the accumulator is zero"),
            MicroOp::InsToPcIfNonNegative => (
                ">=0:ins->pc",
                "Jump only if the accumulator is zero or positive",
            ),
            MicroOp::InsToMc => (
                "ins->mc",
                "Decode: carry on with the micro steps of the instruction in the instruction register",
            ),
            MicroOp::McZero => (
                "mc:=0",
                "End of the instruction: the next micro step starts a new fetch",
            ),
            MicroOp::PcToAb => ("pc->ab", "Put the program counter on the address bus"),
            MicroOp::PcToDb => ("pc->db", "Put the program counter on the data bus"),
            MicroOp::PcInc => ("pc++", "Move the program counter to the next cell"),
            MicroOp::PcIncIfZero => (
                "=0:pc++",
                "Move the program counter on only if the accumulator is zero (skips an instruction)",
            ),
            MicroOp::PcIncIfPositive => (
                ">0:pc++",
                "Move the program counter on only if the accumulator is positive",
            ),
            MicroOp::PcIncIfNegative => (
                "<0:pc++",
                "Move the program counter on only if the accumulator is negative",
            ),
            MicroOp::DbToAb => ("db->ab", "Put the data bus on the address bus"),
            MicroOp::AbToPc => ("ab->pc", "Copy the address bus into the program counter"),
            MicroOp::AccZero => ("acc:=0", "Set the accumulator to zero"),
            MicroOp::AccInc => ("acc++", "Add one to the accumulator"),
            MicroOp::AccDec => ("acc--", "Subtract one from the accumulator"),
            MicroOp::Plus => ("plus", "Add the data bus to the accumulator"),
            MicroOp::Minus => ("minus", "Subtract the data bus from the accumulator"),
            MicroOp::Times => ("times", "Multiply the accumulator by the data bus"),
            MicroOp::Divide => (
                "divide",
                "Divide the accumulator by the data bus; halts if the data bus is zero",
            ),
            MicroOp::AccToDb => ("acc->db", "Copy the accumulator onto the data bus"),
            MicroOp::DbToAcc => ("db->acc", "Copy the data bus into the accumulator"),
            MicroOp::Input => (
                "in->acc",
                "Take the next console input into the accumulator; waits if there is none",
            ),
            MicroOp::Output => ("acc->out", "Print the accumulator on the console"),
            MicroOp::Stop => ("stop", "Halt the machine"),
        }
    }
}

// Workaround for iterating through enums
impl Opcode {
    const ALL: [Opcode; 17] = [
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
        Opcode::Custom,
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
                "Increment the value stored at the memory address given by the operand (it passes through the accumulator)"
            }
            Opcode::Dec => {
                "Decrement the value stored at the memory address given by the operand (it passes through the accumulator)"
            }
            Opcode::Null => {
                "Set the value stored at the memory address given by the operand, and the accumulator, to zero"
            }
            Opcode::Tst => {
                "Load the value at the memory address given by the operand into the accumulator; if it is zero, skip the next instruction"
            }
            Opcode::Halt => "Stop the program and halt execution",
            Opcode::Custom => "Your own list of micro steps",
        }
    }

    // The micro steps that carry the operation out once it is fetched.
    pub const fn micro(self) -> &'static [MicroOp] {
        use MicroOp::*;

        match self {
            Opcode::Nop => &[PcInc, McZero],
            Opcode::Load => &[InsToAb, RamToDb, DbToAcc, PcInc, McZero],
            Opcode::Store => &[InsToAb, AccToDb, DbToRam, PcInc, McZero],
            Opcode::Add => &[InsToAb, RamToDb, Plus, PcInc, McZero],
            Opcode::Sub => &[InsToAb, RamToDb, Minus, PcInc, McZero],
            Opcode::Mul => &[InsToDb, Times, PcInc, McZero],
            Opcode::Div => &[InsToDb, Divide, PcInc, McZero],
            Opcode::Jump => &[InsToPc, McZero],
            Opcode::JumpIfZero => &[PcInc, InsToPcIfZero, McZero],
            Opcode::Input => &[Input, PcInc, McZero],
            Opcode::Output => &[Output, PcInc, McZero],
            Opcode::Inc => &[
                InsToAb, RamToDb, DbToAcc, AccInc, AccToDb, DbToRam, PcInc, McZero,
            ],
            Opcode::Dec => &[
                InsToAb, RamToDb, DbToAcc, AccDec, AccToDb, DbToRam, PcInc, McZero,
            ],
            Opcode::Null => &[InsToAb, AccZero, AccToDb, DbToRam, PcInc, McZero],
            Opcode::Tst => &[InsToAb, RamToDb, DbToAcc, PcIncIfZero, PcInc, McZero],
            Opcode::Halt => &[Stop, McZero],
            Opcode::Custom => &[],
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Definition {
    pub code: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operand: Option<i32>,
    // With a mask, `operand` is the value the decoded operand must match under that mask.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operand_mask: Option<i32>,
    pub mnemonic: String,
    pub opcode: Opcode,

    // Only used by Opcode::Custom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub micro: Vec<MicroOp>,
}

impl Definition {
    pub fn steps(&self) -> &[MicroOp] {
        if self.opcode == Opcode::Custom {
            &self.micro
        } else {
            self.opcode.micro()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SavedSet")]
pub struct InstructionSet {
    pub name: String,
    pub definitions: Vec<Definition>,
    pub format: Format,
}

// Sets saved before there was a format only said whether they were Johnny's.
#[derive(Deserialize)]
struct SavedSet {
    name: String,
    definitions: Vec<Definition>,
    #[serde(default)]
    johnny: bool,
    format: Option<Format>,
}

impl From<SavedSet> for InstructionSet {
    fn from(saved: SavedSet) -> Self {
        let legacy = if saved.johnny {
            Format::johnny()
        } else {
            Format::default()
        };

        Self {
            name: saved.name,
            definitions: saved.definitions,
            format: saved.format.unwrap_or(legacy),
        }
    }
}

impl InstructionSet {
    pub fn find(&self, code: i32) -> Option<&Definition> {
        self.definitions
            .iter()
            .find(|definition| definition.code == code)
    }

    pub fn find_instruction(&self, code: i32, operand: i32) -> Option<&Definition> {
        self.definitions
            .iter()
            .find(|definition| {
                definition.code == code
                    && definition.operand_mask.is_none()
                    && definition.operand == Some(operand)
            })
            .or_else(|| {
                self.definitions.iter().find(|definition| {
                    definition.code == code
                        && definition
                            .operand_mask
                            .zip(definition.operand)
                            .is_some_and(|(mask, expected)| operand & mask == expected & mask)
                })
            })
            .or_else(|| {
                self.definitions
                    .iter()
                    .find(|definition| definition.code == code && definition.operand.is_none())
            })
    }

    pub fn steps(&self, code: i32) -> &[MicroOp] {
        let unknown = match self.format.unknown_code {
            UnknownCode::Nop => Opcode::Nop,
            UnknownCode::Halt => Opcode::Halt,
        };

        self.find(code)
            .map_or(unknown.micro(), Definition::steps)
    }

    pub fn steps_for(&self, code: i32, operand: i32) -> &[MicroOp] {
        let unknown = match self.format.unknown_code {
            UnknownCode::Nop => Opcode::Nop,
            UnknownCode::Halt => Opcode::Halt,
        };

        match self.find_instruction(code, operand) {
            Some(definition) => definition.steps(),
            None if self.find(code).is_some() => unknown.micro(),
            None => self.steps(code),
        }
    }

    pub fn mnemonic_for(&self, code: i32, operand: i32) -> String {
        self.find_instruction(code, operand).map_or_else(
            || format!("{} - ?", self.format.code_text(code)),
            |definition| definition.mnemonic.clone(),
        )
    }

    // The number of operands, which is also what one code is worth in a word.
    fn operands(&self) -> i32 {
        self.format.span(self.format.operand_digits) as i32
    }

    // A word is code * operands + operand: 0x12.34 on the standard machine, 12.345 on Johnny.
    pub fn split(&self, word: i32) -> (i32, i32) {
        (word / self.operands(), word % self.operands())
    }

    pub fn join(&self, code: i32, operand: i32) -> i32 {
        code * self.operands() + operand
    }

    // The largest code, operand and word.
    pub fn limits(&self) -> (i32, i32, i32) {
        let format = &self.format;
        let codes = format.codes.unwrap_or_else(|| format.span(format.code_digits)) as i32;

        (codes - 1, self.operands() - 1, codes * self.operands() - 1)
    }

    pub fn free_code(&self) -> Option<i32> {
        (0..=self.limits().0).find(|code| self.find(*code).is_none())
    }

    pub fn mnemonic(&self, code: i32) -> String {
        self.find(code).map_or_else(
            || format!("{} - ?", self.format.code_text(code)),
            |definition| definition.mnemonic.clone(),
        )
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = read_text(path)?;

        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mc"))
        {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();

            return Self::from_mc(&name, &text);
        }

        let set: Self = serde_json::from_str(&text)?;
        set.check()?;

        Ok(set)
    }

    // The editor clamps a code out of range, which could make it hide another instruction.
    pub fn check(&self) -> anyhow::Result<()> {
        self.format.check()?;

        let max = self.limits().0;

        anyhow::ensure!(
            self.definitions
                .iter()
                .all(|definition| (0..=max).contains(&definition.code)),
            "instruction codes must be between 0 and {max}"
        );

        let max_operand = self.limits().1;
        anyhow::ensure!(
            self.definitions.iter().all(|definition| definition
                .operand
                .is_none_or(|operand| (0..=max_operand).contains(&operand))),
            "fixed instruction operands must be between 0 and {max_operand}"
        );

        anyhow::ensure!(
            self.definitions.iter().all(|definition| {
                definition.operand_mask.is_none_or(|mask| {
                    definition.operand.is_some_and(|operand| operand & mask == operand)
                        && (0..=max_operand).contains(&mask)
                })
            }),
            "operand masks need a fixed operand and must be between 0 and {max_operand}"
        );

        Ok(())
    }

    // A Johnny microcode file: 200 micro step numbers, ten per instruction code, then the names.
    // ponytail: the fetch block (code 0) is not read, the fetch here is fixed.
    pub fn from_mc(name: &str, text: &str) -> anyhow::Result<Self> {
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        let mut set = Self::preset(name, Format::johnny(), &[(0, "NOP", Opcode::Nop)]);

        for code in 1..20 {
            let mut micro = Vec::new();

            // A long instruction may run on into the next block.
            for line in lines.iter().take(200).skip(code * 10) {
                // Blank reads as 0, like in a .ram file.
                let id = match *line {
                    "" => 0,
                    line => line.parse()?,
                };

                let Some(op) = MicroOp::from_johnny(id) else {
                    break;
                };

                micro.push(op);

                if op == MicroOp::McZero {
                    break;
                }
            }

            if !micro.is_empty() {
                let mnemonic = lines.get(200 + code).copied().unwrap_or_default();

                set.definitions.push(Definition {
                    code: code as i32,
                    operand: None,
                    operand_mask: None,
                    mnemonic: if mnemonic.is_empty() {
                        format!("{code:02}")
                    } else {
                        mnemonic.to_string()
                    },
                    opcode: Opcode::Custom,
                    micro,
                });
            }
        }

        Ok(set)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;

        Ok(())
    }
}

impl InstructionSet {
    fn preset(name: &str, format: Format, table: &[(i32, &str, Opcode)]) -> Self {
        Self {
            name: name.to_string(),
            format,

            definitions: table
                .iter()
                .map(|&(code, mnemonic, opcode)| Definition {
                    code,
                    operand: None,
                    operand_mask: None,
                    mnemonic: mnemonic.to_string(),
                    opcode,
                    micro: Vec::new(),
                })
                .collect(),
        }
    }

    // The built-in sets: standard, Johnny, Bonsai, LMC and MARIE.
    pub fn presets() -> [Self; 5] {
        [
            Self::default(),
            Self::johnny(),
            Self::bonsai(),
            Self::lmc(),
            Self::marie(),
        ]
    }

    // The names and codes of the Johnny simulator.
    pub fn johnny() -> Self {
        Self::preset(
            "Johnny",
            Format::johnny(),
            &[
                (0, "NOP", Opcode::Nop),
                (1, "TAKE", Opcode::Load),
                (2, "ADD", Opcode::Add),
                (3, "SUB", Opcode::Sub),
                (4, "SAVE", Opcode::Store),
                (5, "JMP", Opcode::Jump),
                (6, "TST", Opcode::Tst),
                (7, "INC", Opcode::Inc),
                (8, "DEC", Opcode::Dec),
                (9, "NULL", Opcode::Null),
                (10, "HLT", Opcode::Halt),
            ],
        )
    }

    // Johnny's minimal "bonsai" set.
    pub fn bonsai() -> Self {
        Self::preset(
            "Bonsai",
            Format::johnny(),
            &[
                (0, "NOP", Opcode::Nop),
                (1, "INC", Opcode::Inc),
                (2, "DEC", Opcode::Dec),
                (3, "JMP", Opcode::Jump),
                (4, "TST", Opcode::Tst),
                (5, "HLT", Opcode::Halt),
            ],
        )
    }

    pub fn lmc() -> Self {
        use MicroOp::*;

        let mut set = Self::preset(
            "LMC",
            Format::lmc(),
            &[
                (1, "ADD", Opcode::Add),
                (2, "SUB", Opcode::Sub),
                (3, "STA", Opcode::Store),
                (5, "LDA", Opcode::Load),
                (6, "BRA", Opcode::Jump),
                (7, "BRZ", Opcode::JumpIfZero),
            ],
        );

        set.definitions.extend([
            Definition {
                code: 0,
                operand: Some(0),
                operand_mask: None,
                mnemonic: "HLT".to_string(),
                opcode: Opcode::Halt,
                micro: Vec::new(),
            },
            Definition {
                code: 8,
                operand: None,
                operand_mask: None,
                mnemonic: "BRP".to_string(),
                opcode: Opcode::Custom,
                micro: vec![PcInc, InsToPcIfNonNegative, McZero],
            },
            Definition {
                code: 9,
                operand: Some(1),
                operand_mask: None,
                mnemonic: "INP".to_string(),
                opcode: Opcode::Custom,
                micro: vec![Input, PcInc, McZero],
            },
            Definition {
                code: 9,
                operand: Some(2),
                operand_mask: None,
                mnemonic: "OUT".to_string(),
                opcode: Opcode::Custom,
                micro: vec![Output, PcInc, McZero],
            },
        ]);

        set
    }

    pub fn marie() -> Self {
        use MicroOp::*;

        let mut set = Self::preset(
            "MARIE",
            Format::marie(),
            &[
                (1, "Load", Opcode::Load),
                (2, "Store", Opcode::Store),
                (3, "Add", Opcode::Add),
                (4, "Subt", Opcode::Sub),
                (5, "Input", Opcode::Input),
                (6, "Output", Opcode::Output),
                (7, "Halt", Opcode::Halt),
                (9, "Jump", Opcode::Jump),
            ],
        );

        set.definitions.extend([
            Definition {
                code: 0,
                operand: None,
                operand_mask: None,
                mnemonic: "JnS".to_string(),
                opcode: Opcode::Custom,
                micro: vec![PcInc, PcToDb, InsToAb, DbToRam, InsToPc, PcInc, McZero],
            },
            Definition {
                code: 8,
                operand: Some(0x000),
                operand_mask: Some(0xc00),
                mnemonic: "Skipcond <0".to_string(),
                opcode: Opcode::Custom,
                micro: vec![PcIncIfNegative, PcInc, McZero],
            },
            Definition {
                code: 8,
                operand: Some(0x400),
                operand_mask: Some(0xc00),
                mnemonic: "Skipcond =0".to_string(),
                opcode: Opcode::Custom,
                micro: vec![PcIncIfZero, PcInc, McZero],
            },
            Definition {
                code: 8,
                operand: Some(0x800),
                operand_mask: Some(0xc00),
                mnemonic: "Skipcond >0".to_string(),
                opcode: Opcode::Custom,
                micro: vec![PcIncIfPositive, PcInc, McZero],
            },
            Definition {
                code: 10,
                operand: None,
                operand_mask: None,
                mnemonic: "Clear".to_string(),
                opcode: Opcode::Custom,
                micro: vec![AccZero, PcInc, McZero],
            },
            Definition {
                code: 11,
                operand: None,
                operand_mask: None,
                mnemonic: "AddI".to_string(),
                opcode: Opcode::Custom,
                micro: vec![InsToAb, RamToDb, DbToAb, RamToDb, Plus, PcInc, McZero],
            },
            Definition {
                code: 12,
                operand: None,
                operand_mask: None,
                mnemonic: "JumpI".to_string(),
                opcode: Opcode::Custom,
                micro: vec![InsToAb, RamToDb, DbToAb, AbToPc, McZero],
            },
            Definition {
                code: 13,
                operand: None,
                operand_mask: None,
                mnemonic: "LoadI".to_string(),
                opcode: Opcode::Custom,
                micro: vec![InsToAb, RamToDb, DbToAb, RamToDb, DbToAcc, PcInc, McZero],
            },
            Definition {
                code: 14,
                operand: None,
                operand_mask: None,
                mnemonic: "StoreI".to_string(),
                opcode: Opcode::Custom,
                micro: vec![InsToAb, RamToDb, DbToAb, AccToDb, DbToRam, PcInc, McZero],
            },
        ]);

        set
    }
}

impl Default for InstructionSet {
    fn default() -> Self {
        Self::preset(
            "Default",
            Format::default(),
            &[
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
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(name: &str, text: &str) -> anyhow::Result<InstructionSet> {
        let path = std::env::temp_dir().join(format!("vnm-test-{name}"));
        fs::write(&path, text).unwrap();

        let set = InstructionSet::load(&path);
        fs::remove_file(&path).unwrap();

        set
    }

    #[test]
    fn set_files_with_codes_out_of_range_are_refused() {
        let text = r#"{"name":"s","johnny":true,
            "definitions":[{"code":255,"mnemonic":"HLT","opcode":"Halt"}]}"#;

        assert!(load("set.json", text).is_err());
    }

    #[test]
    fn set_files_from_before_the_format_load_by_their_johnny_switch() {
        for (switch, format) in [
            (r#","johnny":true"#, Format::johnny()),
            (r#","johnny":false"#, Format::default()),
            ("", Format::default()),
        ] {
            let text = format!(r#"{{"name":"s"{switch},"definitions":[]}}"#);

            assert_eq!(load("old-set.json", &text).unwrap().format, format, "{switch}");
        }
    }

    #[test]
    fn every_preset_saves_its_format_and_loads_it_back() {
        for preset in InstructionSet::presets() {
            let path = std::env::temp_dir().join(format!("vnm-test-preset-{}.json", preset.name));

            preset.save(&path).unwrap();
            let text = fs::read_to_string(&path).unwrap();
            let set = InstructionSet::load(&path).unwrap();
            fs::remove_file(&path).unwrap();

            assert!(text.contains(r#""format": {"#) && !text.contains(r#""johnny""#));
            assert_eq!(set.format, preset.format);
        }
    }

    #[test]
    fn definitions_with_fixed_operands_are_selected_by_both_fields() {
        let lmc = InstructionSet::lmc();
        let marie = InstructionSet::marie();

        assert_eq!(lmc.mnemonic_for(9, 1), "INP");
        assert_eq!(lmc.mnemonic_for(9, 2), "OUT");
        assert!(lmc.find_instruction(9, 3).is_none());
        assert_eq!(marie.mnemonic_for(8, 0x000), "Skipcond <0");
        assert_eq!(marie.mnemonic_for(8, 0x400), "Skipcond =0");
        assert_eq!(marie.mnemonic_for(8, 0x800), "Skipcond >0");
        assert_eq!(marie.mnemonic_for(8, 0x100), "Skipcond <0");
    }

    #[test]
    fn set_files_with_a_format_the_registers_cannot_hold_are_refused() {
        let text = r#"{"name":"s","format":{"radix":16,"operand_digits":3},"definitions":[]}"#;

        assert!(load("wide.json", text).is_err());
    }

    #[test]
    fn upper_case_mc_files_are_read_as_microcode() {
        assert_eq!(
            load("upper.MC", &"0\r\n".repeat(200)).unwrap().format,
            Format::johnny()
        );
    }

    #[test]
    fn blank_lines_in_a_microcode_file_read_as_zero() {
        let mut lines = vec![""; 200];
        lines[10..15].copy_from_slice(&["4", "2", "18", "9", "7"]);

        let set = InstructionSet::from_mc("blank", &lines.join("\r\n")).unwrap();

        assert_eq!(set.steps(1), Opcode::Load.micro());
    }

    #[test]
    fn a_full_set_has_no_free_code() {
        let mut set = InstructionSet::johnny();

        for code in 11..=19 {
            set.definitions.push(Definition {
                code,
                ..Default::default()
            });
        }

        assert_eq!(set.free_code(), None);
    }
}
