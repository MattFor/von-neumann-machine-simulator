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
                let valid = self.set_memory(state, raw_split);
                self.command_history.push((raw, valid));
            }
            "SetAcc" => {
                let valid = self.set_accumulator(state, raw_split);
                self.command_history.push((raw, valid));
            }
            "SetPc" => {
                let valid = self.set_program_counter(state, raw_split);
                self.command_history.push((raw, valid));
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

    fn set_memory(&self, state: &mut AppState, arguments: Vec<&str>) -> bool {
        if arguments.len() < 3 {
            return false;
        }

        let (Some(address), Some(value)) = (parse_value(arguments[1]), parse_value(arguments[2]))
        else {
            return false;
        };

        if !(0..MEMORY_SIZE as i32).contains(&address) {
            return false;
        }

        state.machine.memory.write(address as usize, value);

        true
    }

    fn set_accumulator(&self, state: &mut AppState, arguments: Vec<&str>) -> bool {
        if arguments.len() < 2 {
            return false;
        }

        let Some(value) = parse_value(arguments[1]) else {
            return false;
        };

        state.machine.cpu.set_acc(value);

        true
    }

    fn set_program_counter(&self, state: &mut AppState, arguments: Vec<&str>) -> bool {
        if arguments.len() < 2 {
            return false;
        }

        let Some(value) = parse_value(arguments[1]) else {
            return false;
        };

        if !(0..MEMORY_SIZE as i32).contains(&value) {
            return false;
        }

        state.machine.cpu.pc = value as u16;

        true
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
            format!("SetMem <ADDRESS [0-{}]> <VALUE DEC/HEX>", MEMORY_SIZE - 1),
            true,
        ));
        self.command_history
            .push(("SetAcc <VALUE DEC/HEX>".to_string(), true));
        self.command_history
            .push((format!("SetPc <ADDRESS [0-{}]>", MEMORY_SIZE - 1), true));
        self.command_history.push(("Reset".to_string(), true));
        self.command_history.push(("Help".to_string(), true));
    }
}
