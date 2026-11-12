use crate::debugger::debug::Debug as DebugState;
use crate::machine::InstructionSet;
use crate::machine::Machine;
use crate::program::{Program, is_ram};
use std::time::Instant;

pub const MIN_SPEED: usize = 1;
pub const MAX_SPEED: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Machine,
    Instructions,
    Guide,

    #[cfg_attr(not(feature = "debug-mode"), allow(dead_code))]
    Debug,
}

pub struct AppState {
    pub mode: UiMode,

    pub machine: Machine,

    // The machine of the other mode (standard or Johnny) and its program name.
    pub other: (Machine, Option<String>),

    // Off shows the simple view, on shows everything.
    pub advanced: bool,

    pub running: bool,
    pub speed: usize,

    pub program_name: Option<String>,

    pub show_memory: bool,
    pub show_registers: bool,
    pub show_control_unit: bool,
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
            other: (Machine::new(), None),

            advanced: false,

            running: false,
            speed: 16,

            program_name: None,

            show_memory: true,
            show_registers: true,
            show_control_unit: true,
            show_cpu: true,
            show_console: true,

            status_message: String::new(),

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
        }
    }

    pub fn status(&self) -> &'static str {
        if self.machine.halted {
            "Halted"
        } else if self.machine.waiting {
            "Waiting for input"
        } else if self.running {
            "Running"
        } else {
            "Ready"
        }
    }

    pub fn johnny(&self) -> bool {
        self.machine.instruction_set.johnny
    }

    // Each mode has its own machine, so switching back and forth loses nothing.
    pub fn toggle_johnny(&mut self) {
        let johnny = self.johnny();

        if self.other.0.instruction_set.johnny == johnny {
            self.other = (Machine::new(), None);

            if !johnny {
                self.other.0.instruction_set = InstructionSet::johnny();
            }
        }

        std::mem::swap(&mut self.machine, &mut self.other.0);
        std::mem::swap(&mut self.program_name, &mut self.other.1);

        self.running = false;
        self.status_message = if johnny {
            "Standard mode"
        } else {
            "Johnny mode: decimal words, 1000 cells"
        }
        .to_string();
    }

    // A set written for the other mode switches to that mode first.
    pub fn use_set(&mut self, set: InstructionSet) {
        if set.johnny != self.johnny() {
            self.toggle_johnny();
        }

        self.status_message = format!("Loaded the {} instruction set", set.name);
        self.machine.instruction_set = set;
    }

    fn load(&mut self, program: Program) {
        if let Some(set) = &program.instruction_set
            && set.johnny != self.johnny()
        {
            self.toggle_johnny();
        }

        program.apply(&mut self.machine);

        self.running = false;
        self.status_message = format!("Loaded {}", program.name);
        self.program_name = Some(program.name);
    }

    pub fn load_example(&mut self) {
        let example = if self.johnny() {
            Program::from_ram("sum", include_str!("../../assets/sum.ram"))
        } else {
            serde_json::from_str(include_str!("../../assets/countdown.json")).map_err(Into::into)
        };

        self.load(example.expect("bundled example is valid"));
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
            .add_filter("Program", &["json", "ram"])
            .pick_file()
        else {
            return;
        };

        match Program::load(&path) {
            Ok(program) => {
                if is_ram(&path) && !self.johnny() {
                    self.toggle_johnny();
                }

                self.load(program);
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
            .add_filter("Johnny program", &["ram"])
            .set_file_name(format!("{name}.json"))
            .save_file()
        else {
            return;
        };

        let name = path
            .file_stem()
            .map_or(name, |stem| stem.to_string_lossy().into_owned());

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
            .add_filter("Instruction set", &["json", "mc"])
            .pick_file()
        else {
            return;
        };

        match InstructionSet::load(&path) {
            Ok(set) => self.use_set(set),

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
