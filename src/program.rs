use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::machine::{InstructionSet, Machine};

pub fn is_ram(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "ram")
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
        let text = fs::read_to_string(path)?;

        if is_ram(path) {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();

            return Self::from_ram(&name, &text);
        }

        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let text = if is_ram(path) {
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
