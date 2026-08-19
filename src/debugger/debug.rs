// NOTE: debug mode is unsafe atp and using commands might crash.
// Use only when you know what you're doing

// Debug mode is not initialized until Debug mode tab is clicked

// To run use feature debug-mode
// example: `cargo run --features debug-mode`

use crate::app::gui_state::AppState;

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
                self.command_history.push((raw.clone(), valid));
            }
            "SetAcc" => {
                let valid = self.set_accumulator(state, raw_split);
                self.command_history.push((raw.clone(), valid));
            }
            "Help" => {
                self.command_history.push((raw.clone(), true));
                self.help()
            }
            _ => {
                self.command_history.push((raw.clone(), false));
            }
        }
    }

    fn set_memory(&self, state: &mut AppState, arguments: Vec<&str>) -> bool {
        if arguments.len() <= 2 {
            return false;
        }

        let address: usize = arguments[1].parse().unwrap();

        let value = if arguments[2].starts_with("0x") {
            i32::from_str_radix(arguments[2].trim_start_matches("0x"), 16).unwrap()
        } else {
            arguments[2].parse::<i32>().unwrap()
        };

	    // Comparison useless due to type limits - TODO: fix
        if address >= 255 || address < 0 {
            return false;
        }

        state.machine.memory.write(address, value);

        true
    }

    // NOTE: can get out of bounds
    fn set_accumulator(&self, state: &mut AppState, arguments: Vec<&str>) -> bool {
        if arguments.len() <= 1 {
            return false;
        }

        let value = arguments[1].parse::<i32>().unwrap();

        state.machine.cpu.acc = value;

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
        self.command_history
            .push(("SetMem <ADDRESS [0-255]> <VALUE DEC/HEX>".to_string(), true));
        self.command_history
            .push(("SetAcc <VALUE DEC>".to_string(), true));
        self.command_history.push(("Help".to_string(), true));
    }
}
