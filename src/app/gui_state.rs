use crate::debugger::debug::Debug as DebugState;
use crate::machine::InstructionSet;
use crate::machine::machine::Machine;
use crate::program::program::Program;
use std::time::Instant;

pub const MIN_SPEED: usize = 1;
pub const MAX_SPEED: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Machine,
    Instructions,

    #[cfg_attr(not(feature = "debug-mode"), allow(dead_code))]
    Debug,
}

pub struct AppState {
    pub mode: UiMode,

    pub machine: Machine,

    pub running: bool,
    pub speed: usize,

    pub program_name: Option<String>,

    pub show_memory: bool,
    pub show_registers: bool,
    pub show_cpu: bool,
    pub show_console: bool,

    pub status_message: String,

    pub debug: Option<DebugState>,

    pub last_tick: std::time::Instant,
    pub time_accumulator: f64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: UiMode::Machine,

            machine: Machine::new(),

            running: false,
            speed: 16,

            program_name: None,

            show_memory: true,
            show_registers: true,
            show_cpu: true,
            show_console: true,

            status_message: "Ready".to_string(),

            debug: None,

            last_tick: Instant::now(),
            time_accumulator: 0.0,
        }
    }

    pub fn halted(&self) -> bool {
        self.machine.halted
    }

    pub fn step(&mut self) {
        self.machine.step();
        self.sync_running();
    }

    pub fn sync_running(&mut self) {
        if self.machine.halted {
            self.running = false;
            self.status_message = "Halted".to_string();
        } else if self.machine.waiting {
            self.running = false;
            self.status_message = "Waiting for input".to_string();
        }
    }

    pub fn reset(&mut self) {
        self.running = false;
        self.machine.reset();
        self.status_message = "Reset".to_string();
    }

    pub fn clear(&mut self) {
        self.running = false;
        self.machine.clear();
        self.program_name = None;
        self.status_message = "Memory cleared".to_string();
    }

    pub fn open_program(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Program", &["json"])
            .pick_file()
        else {
            return;
        };

        match Program::load(&path) {
            Ok(program) => {
                program.apply(&mut self.machine);

                self.running = false;
                self.status_message = format!("Loaded {}", program.name);
                self.program_name = Some(program.name);
            }

            Err(error) => {
                self.status_message = format!("Could not load program: {error}");
            }
        }
    }

    pub fn save_program(&mut self) {
        let name = self
            .program_name
            .clone()
            .unwrap_or_else(|| "program".to_string());

        let Some(path) = rfd::FileDialog::new()
            .add_filter("Program", &["json"])
            .set_file_name(format!("{name}.json"))
            .save_file()
        else {
            return;
        };

        let program = Program::from_machine(name.clone(), &self.machine);

        match program.save(&path) {
            Ok(()) => {
                self.program_name = Some(name);
                self.status_message = format!("Saved to {}", path.display());
            }

            Err(error) => {
                self.status_message = format!("Could not save program: {error}");
            }
        }
    }

    pub fn open_instruction_set(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Instruction set", &["json"])
            .pick_file()
        else {
            return;
        };

        match InstructionSet::load(&path) {
            Ok(set) => {
                self.status_message = format!("Loaded instruction set {}", set.name);
                self.machine.instruction_set = set;
            }

            Err(error) => {
                self.status_message = format!("Could not load instruction set: {error}");
            }
        }
    }

    pub fn save_instruction_set(&mut self) {
        let name = self.machine.instruction_set.name.clone();

        let Some(path) = rfd::FileDialog::new()
            .add_filter("Instruction set", &["json"])
            .set_file_name(format!("{name}.json"))
            .save_file()
        else {
            return;
        };

        match self.machine.instruction_set.save(&path) {
            Ok(()) => {
                self.status_message = format!("Saved to {}", path.display());
            }

            Err(error) => {
                self.status_message = format!("Could not save instruction set: {error}");
            }
        }
    }
}
