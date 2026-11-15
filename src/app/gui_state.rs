use crate::debugger::debug::Debug as DebugState;
use crate::machine::InstructionSet;
use crate::machine::Machine;
use crate::program::{Program, has_extension, is_ram};
use std::path::Path;
use std::time::{Duration, Instant};

pub const MIN_SPEED: usize = 1;
pub const MAX_SPEED: usize = 10_000;

// Name and description of every machine, in the order of InstructionSet::presets.
pub const PRESETS: [(&str, &str); 7] = [
    (
        "Standard",
        "Hex words: a byte of instruction code, then a byte of operand. 256 cells",
    ),
    (
        "Johnny",
        "Like the Johnny simulator: decimal words (two digits of instruction, three of address), \
         1000 cells and the instructions TAKE, ADD, SUB, SAVE, JMP, TST, INC, DEC, NULL, HLT",
    ),
    (
        "Bonsai",
        "Johnny's words with its minimal set: INC, DEC, JMP, TST, HLT",
    ),
    (
        "Little Man Computer",
        "Three decimal digits per mailbox, 100 mailboxes, and values from -999 to 999",
    ),
    (
        "MARIE",
        "16-bit hex words: one digit of instruction, three of address. 4096 cells and \
         two's complement numbers",
    ),
    (
        "CARDIAC",
        "Bell Labs' cardboard computer: a sign and three decimal digits per cell, 100 cells, a \
         four-digit accumulator, and cell 00 fixed at 001, the loader",
    ),
    (
        "SAP-1",
        "Ben Eater's breadboard computer: 16 bytes of a hex digit of instruction and one of \
         address, an 8-bit A register that wraps round, and carry and zero flags for JC and JZ",
    ),
];

// Which preset a set is, if it is one exactly; anything else is Custom.
pub fn preset_of(set: &InstructionSet) -> Option<usize> {
    InstructionSet::presets()
        .iter()
        .position(|preset| preset == set)
}

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

    // The preset whose machine is loaded; editing its set does not make it another one's.
    pub preset: usize,

    // The machines of the other presets with their program names.
    pub others: Vec<(usize, Machine, Option<String>)>,

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
            preset: 0,
            others: Vec::new(),

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

    pub fn back(&mut self) {
        self.running = false;

        if self.machine.back() {
            self.status_message = "Stepped back".to_string();
        }
    }

    pub fn sync_running(&mut self) {
        if self.machine.halted {
            self.running = false;
        }
    }

    // Runs the instructions that came due since the last tick.
    pub fn tick(&mut self, now: Instant) {
        if !self.running || self.machine.waiting {
            return;
        }

        let secs_per_instruction = 1.0 / self.speed as f64;
        // Clamped so a stall (file dialog, waiting for input) is not replayed as a burst. Slow
        // speeds tick once per instruction, a little late, so the clamp is on top of one.
        let dt = (now - self.last_tick)
            .as_secs_f64()
            .min(secs_per_instruction + 0.1);
        self.last_tick = now;

        self.time_accumulator += dt;
        let steps = (self.time_accumulator / secs_per_instruction).floor() as usize;

        if self.machine.run_steps(steps) {
            self.running = false;
            let machine = &self.machine;
            let pc = machine.cpu.pc;

            // Off a breakpoint, the format stopped it on an instruction that kept the pc.
            self.status_message = format!(
                "{} at {}",
                if machine.breakpoints.contains(&pc) {
                    "Breakpoint"
                } else {
                    "Stopped"
                },
                machine.instruction_set.format.operand_text(pc as i32)
            );
        }

        self.time_accumulator -= steps as f64 * secs_per_instruction;

        self.sync_running();
    }

    // How long until the next instruction is due, if one is.
    pub fn due(&self) -> Option<Duration> {
        let left = 1.0 / self.speed as f64 - self.time_accumulator;

        (self.running && !self.machine.waiting).then(|| Duration::from_secs_f64(left.max(0.0)))
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

    // Everything that decides which panels are shown and where.
    pub fn layout(&self) -> impl PartialEq + use<> {
        (
            self.mode,
            self.advanced,
            [
                self.show_memory,
                self.show_registers,
                self.show_control_unit,
                self.show_cpu,
                self.show_console,
            ],
            self.preset,
            self.machine.instruction_set.format,
        )
    }

    // Each preset has its own machine, so switching back and forth loses nothing.
    pub fn switch(&mut self, preset: usize) {
        if preset == self.preset {
            return;
        }

        let (machine, name) = match self.others.iter().position(|(slot, ..)| *slot == preset) {
            Some(index) => {
                let (_, machine, name) = self.others.remove(index);

                (machine, name)
            }

            None => {
                let mut machine = Machine::new();
                machine.instruction_set = InstructionSet::presets()[preset].clone();

                (machine, None)
            }
        };

        let machine = std::mem::replace(&mut self.machine, machine);
        let name = std::mem::replace(&mut self.program_name, name);

        self.others.push((self.preset, machine, name));
        self.preset = preset;

        self.running = false;
        self.status_message = format!("{} machine", PRESETS[preset].0);
    }

    // The picker: another preset's machine as it was left, or this one's set back if it was changed.
    pub fn pick(&mut self, preset: usize) {
        if preset != self.preset {
            self.switch(preset);
        } else if preset_of(&self.machine.instruction_set).is_none() {
            self.machine.instruction_set = InstructionSet::presets()[preset].clone();
            self.running = false;
            self.status_message = format!("Restored the {} instruction set", PRESETS[preset].0);
        }
    }

    // A preset's set goes to that preset's machine, any other set to a machine of its format.
    fn route(&mut self, set: &InstructionSet) {
        let presets = InstructionSet::presets();
        let target = preset_of(set).or_else(|| {
            (set.format != self.machine.instruction_set.format)
                .then(|| presets.iter().position(|preset| preset.format == set.format))
                .flatten()
        });

        if let Some(preset) = target {
            self.switch(preset);
        }
    }

    pub fn use_set(&mut self, set: InstructionSet) {
        self.route(&set);

        self.status_message = format!("Loaded the {} instruction set", set.name);
        self.machine.instruction_set = set;
    }

    pub fn load(&mut self, program: Program) {
        if let Some(set) = &program.instruction_set {
            self.route(set);
        }

        program.apply(&mut self.machine);

        self.running = false;
        self.status_message = format!("Loaded {}", program.name);
        self.program_name = Some(program.name);
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
            .add_filter("Program", &["json", "ram", "asm", "mas"])
            .pick_file()
        else {
            return;
        };

        self.open_program_at(&path);
    }

    pub fn open_program_at(&mut self, path: &Path) {
        match Program::load(path, &self.machine.instruction_set) {
            Ok(program) => {
                // Johnny's files go to Johnny even from Bonsai, whose words are the same.
                if is_ram(path) {
                    self.route(&InstructionSet::johnny());
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
            .add_filter("Assembly", &["asm"])
            .add_filter("MARIE assembly", &["mas"])
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

        self.open_set_at(&path);
    }

    pub fn open_set_at(&mut self, path: &Path) {
        match InstructionSet::load(path) {
            Ok(set) => {
                if has_extension(path, "mc") {
                    self.route(&InstructionSet::johnny());
                }

                self.use_set(set);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::examples;

    #[test]
    fn a_set_that_matches_no_preset_is_custom() {
        for (index, preset) in InstructionSet::presets().into_iter().enumerate() {
            assert_eq!(preset_of(&preset), Some(index), "{}", preset.name);
        }

        let mut renamed = InstructionSet::lmc();
        renamed.definitions[0].mnemonic = "PLUS".to_string();
        let mut smaller = InstructionSet::marie();
        smaller.format.cells = Some(100);

        assert_eq!(preset_of(&renamed), None);
        assert_eq!(preset_of(&smaller), None);
    }

    #[test]
    fn every_preset_keeps_its_own_machine_and_changed_set() {
        let mut state = AppState::new();
        let sum = examples()
            .into_iter()
            .find(|example| example.name == "Johnny sum")
            .unwrap();

        state.load(sum.program);

        assert_eq!((state.preset, state.machine.memory.read(0)), (1, 1010));

        state.pick(3);

        assert_eq!(state.machine.instruction_set, InstructionSet::lmc());
        assert_eq!(state.machine.memory.read(0), 0);
        assert_eq!(state.program_name, None);

        state.machine.memory.write(0, 902);
        state.machine.instruction_set.definitions[0].mnemonic = "PLUS".to_string();
        state.pick(1);

        assert_eq!(state.machine.memory.read(0), 1010);
        assert_eq!(state.program_name.as_deref(), Some("sum"));

        state.pick(3);

        assert_eq!(state.machine.memory.read(0), 902);
        assert_eq!(preset_of(&state.machine.instruction_set), None);

        // Picked again, the machine gets its set back and keeps its memory.
        state.pick(3);

        assert_eq!(state.machine.instruction_set, InstructionSet::lmc());
        assert_eq!(state.machine.memory.read(0), 902);
        assert_eq!(state.others.len(), 2);
    }

    #[test]
    fn a_set_of_another_format_goes_to_a_machine_of_that_format() {
        let mut state = AppState::new();
        state.machine.memory.write(0, 0x0a0b);

        let microcode = InstructionSet::from_mc("mine", &"0\n".repeat(200)).unwrap();
        state.use_set(microcode);

        assert_eq!(state.preset, 1);
        assert_eq!(state.machine.instruction_set.name, "mine");

        state.pick(0);

        assert_eq!(state.machine.memory.read(0), 0x0a0b);
    }

    #[test]
    fn johnny_files_opened_on_bonsai_go_to_johnny_and_leave_bonsai_alone() {
        let ram = std::env::temp_dir().join("vnm-test-bonsai.ram");
        let mc = std::env::temp_dir().join("vnm-test-bonsai.mc");
        std::fs::write(&ram, include_str!("../../assets/sum.ram")).unwrap();
        std::fs::write(&mc, "0\n".repeat(200)).unwrap();

        for path in [&ram, &mc] {
            let mut state = AppState::new();
            state.pick(2);
            state.machine.memory.write(0, 5);

            if is_ram(path) {
                state.open_program_at(path);
            } else {
                state.open_set_at(path);
            }

            assert_eq!(state.preset, 1, "{path:?}");

            state.pick(2);

            assert_eq!(state.machine.memory.read(0), 5);
            assert_eq!(state.machine.instruction_set, InstructionSet::presets()[2]);
        }

        std::fs::remove_file(&ram).unwrap();
        std::fs::remove_file(&mc).unwrap();
    }

    #[test]
    fn a_slow_run_ticks_once_per_instruction_at_the_same_speed() {
        let mut state = AppState::new();
        state.speed = 1;
        state.running = true;
        let start = state.last_tick;

        state.tick(start + Duration::from_millis(400));

        let due = state.due().unwrap();

        assert_eq!(state.machine.cpu.pc, 0);
        assert!((Duration::from_millis(590)..=Duration::from_millis(600)).contains(&due));

        // Woken when it is due, the instruction runs; a long stall is not replayed as a burst.
        state.tick(start + Duration::from_millis(1001));

        assert_eq!(state.machine.cpu.pc, 1);

        state.tick(start + Duration::from_secs(10));

        assert_eq!(state.machine.cpu.pc, 2);

        state.back();

        assert_eq!((state.machine.cpu.pc, state.running, state.due()), (1, false, None));

        // Frames that come a little after the instruction was due do not slow the run down.
        let mut state = AppState::new();
        state.speed = 10;
        state.running = true;
        let start = state.last_tick;

        for frame in 1..=100 {
            state.tick(start + Duration::from_millis(108 * frame));
        }

        assert!((107..=108).contains(&state.machine.cpu.pc), "{}", state.machine.cpu.pc);
    }

    #[test]
    fn a_mas_file_switches_to_the_marie_machine() {
        let path = std::env::temp_dir().join("vnm-test-switch.mas");
        std::fs::write(&path, "Input\nOutput\nHalt\n").unwrap();

        let mut state = AppState::new();
        let program = Program::load(&path, &state.machine.instruction_set).unwrap();
        std::fs::remove_file(&path).unwrap();
        state.load(program);

        assert_eq!(PRESETS[state.preset].0, "MARIE");
        assert_eq!(state.machine.instruction_set, InstructionSet::marie());
        assert_eq!(state.machine.memory.data()[..3], [0x5000, 0x6000, 0x7000]);
    }
}
