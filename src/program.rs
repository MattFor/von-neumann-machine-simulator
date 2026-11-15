use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::assembler::{assemble, disassemble};
use crate::machine::{DataEncoding, InstructionSet, MEMORY_SIZE, Machine};
use crate::utilities::read_text;

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

pub fn is_ram(path: &Path) -> bool {
    has_extension(path, "ram")
}

// The first mistake is enough for the status bar.
fn assembled(text: &str, set: &InstructionSet) -> anyhow::Result<(Vec<i32>, u16)> {
    assemble(text, set).map_err(|errors| match errors.len() {
        1 => anyhow::anyhow!("{}", errors[0]),
        count => anyhow::anyhow!("{}, and {} more", errors[0], count - 1),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub name: String,

    pub entry_point: u16,

    pub memory: Vec<i32>,

    // Absent in older files; those keep whatever set is loaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_set: Option<InstructionSet>,
}

// Kept available for the example selector added in the next UI change.
#[allow(dead_code)]
pub struct Example {
    pub name: &'static str,
    pub preset: InstructionSet,
    pub program: Program,
}

#[allow(dead_code)]
pub fn examples() -> Vec<Example> {
    fn example(
        name: &'static str,
        preset: InstructionSet,
        mut program: Program,
    ) -> Example {
        program.instruction_set = Some(preset.clone());

        Example {
            name,
            preset,
            program,
        }
    }

    vec![
        example(
            "Standard countdown",
            InstructionSet::default(),
            serde_json::from_str(include_str!("../assets/countdown.json"))
                .expect("bundled Standard example is valid"),
        ),
        example(
            "Johnny sum",
            InstructionSet::johnny(),
            Program::from_ram("sum", include_str!("../assets/sum.ram"))
                .expect("bundled Johnny example is valid"),
        ),
        example(
            "Bonsai increment",
            InstructionSet::bonsai(),
            Program::from_ram("bonsai", include_str!("../assets/bonsai.ram"))
                .expect("bundled Bonsai example is valid"),
        ),
        example(
            "LMC echo",
            InstructionSet::lmc(),
            serde_json::from_str(include_str!("../assets/lmc-echo.json"))
                .expect("bundled LMC example is valid"),
        ),
        example(
            "MARIE echo",
            InstructionSet::marie(),
            serde_json::from_str(include_str!("../assets/marie-echo.json"))
                .expect("bundled MARIE example is valid"),
        ),
    ]
}

impl Program {
    pub fn from_machine(name: String, machine: &Machine) -> Self {
        let data = machine.memory.data();

        let length = data
            .iter()
            .rposition(|value| *value != 0)
            .map_or(0, |index| index + 1);

        Self {
            name,
            entry_point: machine.entry_point,
            memory: data[..length].to_vec(),
            instruction_set: Some(machine.instruction_set.clone()),
        }
    }

    // A Johnny program file: one decimal word per line.
    pub fn from_ram(name: &str, text: &str) -> anyhow::Result<Self> {
        let memory = text
            .lines()
            .map(|line| match line.trim() {
                "" => Ok(0),
                word => word.parse(),
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            name: name.to_string(),
            entry_point: 0,
            memory,
            instruction_set: None,
        })
    }

    // Johnny has exactly 1000 cells.
    pub fn to_ram(&self) -> String {
        let words: Vec<String> = (0..1000)
            .map(|address| self.memory.get(address).copied().unwrap_or(0).to_string())
            .collect();

        words.join("\r\n")
    }

    // Files ending in .ram are Johnny programs, .asm are text in the mnemonics of `set`,
    // everything else is JSON.
    pub fn load(path: &Path, set: &InstructionSet) -> anyhow::Result<Self> {
        let text = read_text(path)?;
        let name = path.file_stem().unwrap_or_default().to_string_lossy();

        let program: Self = if is_ram(path) {
            Self::from_ram(&name, &text)?
        } else if has_extension(path, "asm") {
            let (memory, entry_point) = assembled(&text, set)?;

            Self {
                name: name.to_string(),
                entry_point,
                memory,
                instruction_set: None,
            }
        } else {
            serde_json::from_str(&text)?
        };

        // A word wider than the 16-bit registers would run as another instruction than it shows.
        anyhow::ensure!(
            program.memory.len() <= MEMORY_SIZE
                && program
                    .memory
                    .iter()
                    .all(|word| u16::try_from(*word).is_ok()),
            "a program is at most {MEMORY_SIZE} words, each between 0 and 65535"
        );

        if let Some(set) = &program.instruction_set {
            set.check()?;

            if set.format.data_encoding == DataEncoding::Lmc {
                anyhow::ensure!(
                    program.memory.len() <= set.format.cells()
                        && program.memory.iter().all(|word| (0..=999).contains(word)),
                    "an LMC program has at most {} cells, each between 000 and 999",
                    set.format.cells()
                );
            }
        }

        Ok(program)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let text = if is_ram(path) {
            // Johnny reads decimal words, so another format would come back as another program.
            anyhow::ensure!(
                self.instruction_set
                    .as_ref()
                    .is_none_or(|set| set.format.johnny_compatible()),
                "a .ram file holds Johnny's words: radix 10, 2 code digits and 3 operand digits"
            );

            self.to_ram()
        } else if has_extension(path, "asm") {
            let set = self
                .instruction_set
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("an .asm file needs the instruction set"))?;
            let text = disassemble(&self.memory, self.entry_point, set);
            // Words or cells the format cannot hold would not load back.
            let (memory, entry_point) = assembled(&text, set)?;
            let length = self
                .memory
                .iter()
                .rposition(|word| *word != 0)
                .map_or(0, |index| index + 1);

            anyhow::ensure!(
                (&memory[..], entry_point) == (&self.memory[..length], self.entry_point),
                "this program would not read back the same from an .asm file"
            );

            text
        } else {
            serde_json::to_string_pretty(self)?
        };

        fs::write(path, text)?;

        Ok(())
    }

    pub fn apply(&self, machine: &mut Machine) {
        machine.memory.load(&self.memory);
        machine.entry_point = self.entry_point;
        machine.breakpoints.clear();

        if let Some(set) = &self.instruction_set {
            machine.instruction_set = set.clone();
        }

        machine.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{Format, PcOverflow};

    // A file of its own per test, since tests run in parallel.
    fn load(name: &str, text: &str) -> anyhow::Result<Program> {
        let path = std::env::temp_dir().join(format!("vnm-test-{name}"));
        fs::write(&path, text).unwrap();

        let program = Program::load(&path, &InstructionSet::default());
        fs::remove_file(&path).unwrap();

        program
    }

    #[test]
    fn words_that_do_not_fit_sixteen_bits_are_refused() {
        for memory in ["[-1]", "[70000]"] {
            let text = format!(r#"{{"name":"x","entry_point":0,"memory":{memory}}}"#);

            assert!(load("word.json", &text).is_err());
        }

        assert!(load("negative.ram", "1010\r\n-5\r\n").is_err());
        assert!(load("fits.ram", "1010\r\n65535\r\n").is_ok());
    }

    #[test]
    fn programs_larger_than_the_memory_are_refused() {
        assert!(load("long.ram", &"0\n".repeat(MEMORY_SIZE + 1)).is_err());
    }

    #[test]
    fn instruction_codes_out_of_range_are_refused() {
        let text = r#"{"name":"x","entry_point":0,"memory":[],"instruction_set":
            {"name":"s","definitions":[{"code":300,"mnemonic":"X","opcode":"Nop"}]}}"#;

        assert!(load("code.json", text).is_err());
    }

    #[test]
    fn upper_case_ram_files_are_read_as_johnny_programs() {
        let program = load("upper.RAM", "1010\r\n2011\r\n").unwrap();

        assert_eq!(program.memory, [1010, 2011]);
    }

    #[test]
    fn only_a_johnny_program_is_saved_as_ram() {
        let path = std::env::temp_dir().join("vnm-test-standard.ram");
        let mut machine = Machine::new();

        let saved = Program::from_machine("x".to_string(), &machine).save(&path);
        let written = fs::remove_file(&path).is_ok();

        assert!(saved.is_err() && !written);

        machine.instruction_set = InstructionSet::johnny();
        Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap();
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn ram_files_need_a_johnny_compatible_format() {
        let path = std::env::temp_dir().join("vnm-test-compatible.ram");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        machine.instruction_set.format.pc_overflow = PcOverflow::Halt;

        Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap();
        fs::remove_file(&path).unwrap();

        machine.instruction_set.format.radix = 8;
        let error = Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap_err();

        assert!(error.to_string().contains("radix 10"));
    }

    #[test]
    fn programs_from_before_the_format_load_by_their_johnny_switch() {
        let text = r#"{"name":"x","entry_point":0,"memory":[1010],"instruction_set":
            {"name":"Johnny","johnny":true,"definitions":[]}}"#;
        let program = load("old-program.json", text).unwrap();

        assert_eq!(program.instruction_set.unwrap().format, Format::johnny());
    }

    #[test]
    fn a_saved_program_keeps_its_format() {
        let path = std::env::temp_dir().join("vnm-test-format.json");
        let mut machine = Machine::new();
        machine.instruction_set.format = Format {
            radix: 8,
            code_digits: 1,
            operand_digits: 3,
            acc_min: -999,
            pc_overflow: PcOverflow::Halt,
            ..Format::default()
        };
        machine
            .instruction_set
            .definitions
            .retain(|definition| definition.code < 8);

        Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let program = Program::load(&path, &machine.instruction_set).unwrap();
        fs::remove_file(&path).unwrap();

        assert!(text.contains(r#""radix": 8"#));
        assert_eq!(
            program.instruction_set.unwrap().format,
            machine.instruction_set.format
        );
    }

    #[test]
    fn huge_files_are_refused_before_they_are_read() {
        let path = std::env::temp_dir().join("vnm-test-huge.json");
        fs::File::create(&path)
            .unwrap()
            .set_len(16 * 1024 * 1024 + 1)
            .unwrap();

        let error = Program::load(&path, &InstructionSet::default()).unwrap_err();
        fs::remove_file(&path).unwrap();

        assert!(error.to_string().contains("larger"));
    }

    #[test]
    fn asm_files_are_written_and_read_in_the_loaded_set() {
        let path = std::env::temp_dir().join("vnm-test-sum.ASM");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        Program::from_ram("sum", include_str!("../assets/sum.ram"))
            .unwrap()
            .apply(&mut machine);
        machine.entry_point = 2;

        let saved = Program::from_machine("sum".to_string(), &machine);
        saved.save(&path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let program = Program::load(&path, &machine.instruction_set).unwrap();
        fs::remove_file(&path).unwrap();

        assert!(text.starts_with("TAKE 10\nADD 11\nSAVE 12\nHLT 0\n"));
        assert!(text.ends_with("NOP 7\nEND 2\n"));
        assert_eq!(program.memory, saved.memory);
        assert_eq!(program.entry_point, 2);
        assert!(program.instruction_set.is_none());
    }

    #[test]
    fn asm_files_report_the_first_mistake_and_how_many_more() {
        let error = load("mistakes.asm", "LOAD 1\nFOO\nBAR\n").unwrap_err();

        assert_eq!(error.to_string(), "line 2: unknown mnemonic FOO, and 1 more");
    }

    #[test]
    fn memory_the_format_cannot_hold_is_not_saved_as_asm() {
        let path = std::env::temp_dir().join("vnm-test-wide.asm");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        machine.memory.write(0, 65535);

        let error = Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap_err();
        let written = fs::remove_file(&path).is_ok();

        assert!(error.to_string().starts_with("line 1: 65535 is out of range"));
        assert!(!written);
    }
}
