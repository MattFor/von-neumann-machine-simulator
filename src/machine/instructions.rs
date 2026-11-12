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
    InsToMc,
    McZero,
    PcToAb,
    PcInc,
    PcIncIfZero,
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
    pub const ALL: [MicroOp; 24] = [
        MicroOp::DbToRam,
        MicroOp::RamToDb,
        MicroOp::DbToIns,
        MicroOp::InsToAb,
        MicroOp::InsToDb,
        MicroOp::InsToPc,
        MicroOp::InsToPcIfZero,
        MicroOp::InsToMc,
        MicroOp::McZero,
        MicroOp::PcToAb,
        MicroOp::PcInc,
        MicroOp::PcIncIfZero,
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
            MicroOp::InsToMc => (
                "ins->mc",
                "Decode: carry on with the micro steps of the instruction in the instruction register",
            ),
            MicroOp::McZero => (
                "mc:=0",
                "End of the instruction: the next micro step starts a new fetch",
            ),
            MicroOp::PcToAb => ("pc->ab", "Put the program counter on the address bus"),
            MicroOp::PcInc => ("pc++", "Move the program counter to the next cell"),
            MicroOp::PcIncIfZero => (
                "=0:pc++",
                "Move the program counter on only if the accumulator is zero (skips an instruction)",
            ),
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
pub struct InstructionSet {
    pub name: String,
    pub definitions: Vec<Definition>,

    // Johnny words: decimal, two digits of code then three of operand.
    #[serde(default)]
    pub johnny: bool,
}

impl InstructionSet {
    pub fn find(&self, code: i32) -> Option<&Definition> {
        self.definitions
            .iter()
            .find(|definition| definition.code == code)
    }

    // Unknown codes behave like NOP.
    pub fn steps(&self, code: i32) -> &[MicroOp] {
        self.find(code)
            .map_or(Opcode::Nop.micro(), Definition::steps)
    }

    // A word is 0x1234 (code 0x12, operand 0x34), or 12.345 for Johnny.
    pub fn split(&self, word: i32) -> (i32, i32) {
        if self.johnny {
            (word / 1000, word % 1000)
        } else {
            ((word >> 8) & 0xff, word & 0xff)
        }
    }

    pub fn join(&self, code: i32, operand: i32) -> i32 {
        if self.johnny {
            code * 1000 + operand
        } else {
            ((code & 0xff) << 8) | (operand & 0xff)
        }
    }

    // The largest code, operand and word.
    pub fn limits(&self) -> (i32, i32, i32) {
        if self.johnny {
            (19, 999, 19_999)
        } else {
            (255, 255, 0xffff)
        }
    }

    // Numbers read as hex, or as decimal for Johnny.
    pub fn digits(&self, value: i32, hex: usize, decimal: usize) -> String {
        if self.johnny {
            format!("{value:0decimal$}")
        } else {
            format!("{value:0hex$X}")
        }
    }

    pub fn free_code(&self) -> i32 {
        (0..=self.limits().0)
            .find(|code| self.find(*code).is_none())
            .unwrap_or(0)
    }

    pub fn mnemonic(&self, code: i32) -> String {
        self.find(code).map_or_else(
            || format!("{} - ?", self.digits(code, 2, 2)),
            |definition| definition.mnemonic.clone(),
        )
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;

        if path.extension().is_some_and(|extension| extension == "mc") {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();

            return Self::from_mc(&name, &text);
        }

        Ok(serde_json::from_str(&text)?)
    }

    // A Johnny microcode file: 200 micro step numbers, ten per instruction code, then the names.
    // ponytail: the fetch block (code 0) is not read, the fetch here is fixed.
    pub fn from_mc(name: &str, text: &str) -> anyhow::Result<Self> {
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        let mut set = Self::preset(name, true, &[(0, "NOP", Opcode::Nop)]);

        for code in 1..20 {
            let mut micro = Vec::new();

            // A long instruction may run on into the next block.
            for line in lines.iter().take(200).skip(code * 10) {
                let Some(op) = MicroOp::from_johnny(line.parse()?) else {
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
    fn preset(name: &str, johnny: bool, table: &[(i32, &str, Opcode)]) -> Self {
        Self {
            name: name.to_string(),
            johnny,

            definitions: table
                .iter()
                .map(|&(code, mnemonic, opcode)| Definition {
                    code,
                    mnemonic: mnemonic.to_string(),
                    opcode,
                    micro: Vec::new(),
                })
                .collect(),
        }
    }

    // The names and codes of the Johnny simulator.
    pub fn johnny() -> Self {
        Self::preset(
            "Johnny",
            true,
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
            true,
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
}

impl Default for InstructionSet {
    fn default() -> Self {
        Self::preset(
            "Default",
            false,
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
