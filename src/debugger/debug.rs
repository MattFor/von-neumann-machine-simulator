use crate::app::gui_state::AppState;
use crate::machine::MEMORY_SIZE;
use crate::utilities::parse_value;

pub struct Debug {
    pub command_history: Vec<(String, bool)>,
    pub command_input: String,
}

impl Debug {
    pub fn new() -> Self {
        Self {
            command_history: Vec::new(),
            command_input: String::new(),
        }
    }

    pub fn send(&mut self, state: &mut AppState, raw: String) {
        let raw_split: Vec<&str> = raw.split_whitespace().collect();

        if raw_split.is_empty() {
            return;
        }

        let command = raw_split[0];

        match command {
            "SetMem" => {
                let result = self.set_memory(state, raw_split);
                let (line, valid) = match result {
                    Ok((address, cell)) => (Self::address_reply(raw, address, cell), true),
                    Err(Some(cell)) => (format!("{raw} (cell {cell} is fixed)"), false),
                    Err(None) => (raw, false),
                };
                self.command_history.push((line, valid));
            }
            "SetAcc" => {
                let valid = self.set_accumulator(state, raw_split);
                self.command_history.push((raw, valid));
            }
            "SetPc" => {
                let result = self.set_program_counter(state, raw_split);
                let (line, valid) = match result {
                    Some((address, cell)) => (Self::address_reply(raw, address, cell), true),
                    None => (raw, false),
                };
                self.command_history.push((line, valid));
            }
            "Reset" => {
                state.reset();
                self.command_history.push((raw, true));
            }
            "Help" => {
                self.command_history.push((raw, true));
                self.help()
            }
            _ => {
                self.command_history.push((raw, false));
            }
        }
    }

    // Err(Some(cell)) when the format fixes that cell.
    fn set_memory(
        &self,
        state: &mut AppState,
        arguments: Vec<&str>,
    ) -> Result<(i32, i32), Option<i32>> {
        if arguments.len() < 3 {
            return Err(None);
        }

        let (Some(address), Some(value)) = (parse_value(arguments[1]), parse_value(arguments[2]))
        else {
            return Err(None);
        };

        if !(0..MEMORY_SIZE as i32).contains(&address) {
            return Err(None);
        }

        let format = state.machine.instruction_set.format;
        let cell = address.rem_euclid(format.cells().max(1) as i32);

        if format.fixed().is_some_and(|(fixed, _)| fixed == cell as usize) {
            return Err(Some(cell));
        }

        state.machine.memory.write(cell as usize, value);

        Ok((address, cell))
    }

    fn set_accumulator(&self, state: &mut AppState, arguments: Vec<&str>) -> bool {
        if arguments.len() < 2 {
            return false;
        }

        let Some(value) = parse_value(arguments[1]) else {
            return false;
        };

        state.machine.set_acc(value);

        true
    }

    fn set_program_counter(&self, state: &mut AppState, arguments: Vec<&str>) -> Option<(i32, i32)> {
        if arguments.len() < 2 {
            return None;
        }

        let value = parse_value(arguments[1])?;

        if !(0..MEMORY_SIZE as i32).contains(&value) {
            return None;
        }

        let cells = state.machine.instruction_set.format.cells().max(1) as i32;
        let cell = value.rem_euclid(cells);
        state.machine.cpu.pc = cell as u16;

        Some((value, cell))
    }

    fn address_reply(raw: String, address: i32, cell: i32) -> String {
        if address == cell {
            raw
        } else {
            format!("{raw} (address {address} reduced to {cell})")
        }
    }

    pub fn help(&mut self) {
        self.command_history.push((
            "WARNING! Debug mode is unsafe at this point!".to_string(),
            false,
        ));

        self.command_history.push((
            "Use only when you know what you're doing".to_string(),
            false,
        ));

        self.command_history.push((" ".to_string(), false));
        self.command_history
            .push(("Available Commands:".to_string(), true));
        self.command_history.push((
            format!(
                "SetMem <ADDRESS [0-{}], reduced modulo format cells> <VALUE DEC/HEX>",
                MEMORY_SIZE - 1
            ),
            true,
        ));
        self.command_history
            .push(("SetAcc <VALUE DEC/HEX>".to_string(), true));
        self.command_history
            .push((
                format!(
                    "SetPc <ADDRESS [0-{}], reduced modulo format cells>",
                    MEMORY_SIZE - 1
                ),
                true,
            ));
        self.command_history.push(("Reset".to_string(), true));
        self.command_history.push(("Help".to_string(), true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::gui_state::AppState;
    use crate::machine::InstructionSet;

    #[test]
    fn setmem_and_setpc_reduce_addresses_to_the_format_cells() {
        let mut state = AppState::new();
        state.machine.instruction_set = InstructionSet::lmc();
        let mut debug = Debug::new();

        debug.send(&mut state, "SetMem 500 17".to_string());
        debug.send(&mut state, "SetPc 105".to_string());

        assert_eq!(state.machine.memory.read(0), 17);
        assert_eq!(state.machine.memory.read(500), 0);
        assert_eq!(state.machine.cpu.pc, 5);
        assert_eq!(
            debug.command_history,
            [
                ("SetMem 500 17 (address 500 reduced to 0)".to_string(), true),
                ("SetPc 105 (address 105 reduced to 5)".to_string(), true),
            ]
        );
    }

    #[test]
    fn setmem_refuses_a_fixed_cell() {
        let mut state = AppState::new();
        state.machine.instruction_set = InstructionSet::cardiac();
        let mut debug = Debug::new();

        debug.send(&mut state, "SetMem 100 5".to_string());

        assert_eq!(state.machine.memory.read(0), 0);
        assert_eq!(
            debug.command_history,
            [("SetMem 100 5 (cell 0 is fixed)".to_string(), false)]
        );
    }
}
