use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::machine::{InstructionSet, MEMORY_SIZE, Machine};
use crate::utilities::read_text;

pub fn is_ram(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("ram"))
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

    // Files ending in .ram are Johnny programs, everything else is JSON.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = read_text(path)?;

        let program: Self = if is_ram(path) {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();

            Self::from_ram(&name, &text)?
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
        }

        Ok(program)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let text = if is_ram(path) {
            // Johnny reads the words as decimal, so a standard program would come back as another.
            anyhow::ensure!(
                self.instruction_set.as_ref().is_none_or(|set| set.johnny),
                "only a Johnny mode program can be saved as .ram"
            );

            self.to_ram()
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

    // A file of its own per test, since tests run in parallel.
    fn load(name: &str, text: &str) -> anyhow::Result<Program> {
        let path = std::env::temp_dir().join(format!("vnm-test-{name}"));
        fs::write(&path, text).unwrap();

        let program = Program::load(&path);
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
    fn huge_files_are_refused_before_they_are_read() {
        let path = std::env::temp_dir().join("vnm-test-huge.json");
        fs::File::create(&path)
            .unwrap()
            .set_len(16 * 1024 * 1024 + 1)
            .unwrap();

        let error = Program::load(&path).unwrap_err();
        fs::remove_file(&path).unwrap();

        assert!(error.to_string().contains("larger"));
    }
}
