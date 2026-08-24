use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::machine::machine::Machine;

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub name: String,

    pub entry_point: u16,

    pub memory: Vec<i32>,
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
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;

        Ok(())
    }

    pub fn apply(&self, machine: &mut Machine) {
        machine.memory.load(&self.memory);
        machine.entry_point = self.entry_point;
        machine.reset();
    }
}
