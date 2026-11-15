mod executor;
mod format;
mod instructions;
mod memory;
mod registers;

use std::collections::{HashSet, VecDeque};

pub use format::{DataEncoding, Format, PcOverflow, UnknownCode};
pub use instructions::{Definition, FETCH, InstructionSet, MicroOp, Opcode};
pub use memory::MEMORY_SIZE;
pub use registers::{ACC_MAX, ACC_MIN};

use executor::execute;
use memory::Memory;
use registers::Registers;

// Output is ASCII only, so cutting at a byte offset is safe.
const OUTPUT_LIMIT: usize = 16 * 1024;

// More micro steps than this in one instruction means its micro program loops.
const MICRO_LIMIT: usize = 1000;

// How many steps Back can undo.
const HISTORY_LIMIT: usize = 4096;

// What a step changed besides the registers, so that Back can put it back.
enum Change {
    Cell(usize, i32), // a memory cell and the word it held
    Input(i32),       // a value IN took from the queue
    Output(String),   // the console before the limit cut it short
}

// The machine before a step, apart from the changes it logs.
struct Undo {
    cpu: Registers,
    previous: Registers,
    halted: bool,
    waiting: bool,
    executing: bool,
    mc: usize,
    output: usize,
    recorded: bool,
    changes: Vec<Change>,
}

// Why Machine::run ended.
pub enum Stop {
    Halted,
    StepLimit,
    NoInput,
}

pub struct Machine {
    pub cpu: Registers,
    pub previous: Registers, // registers before the last step
    pub memory: Memory,

    pub instruction_set: InstructionSet,

    pub entry_point: u16,

    pub halted: bool,
    pub waiting: bool,

    // Control unit: fetching or executing, and the index of the next micro step.
    pub executing: bool,
    pub mc: usize,

    // Micro steps fired by hand since recording was switched on.
    pub recording: Option<Vec<MicroOp>>,

    pub breakpoints: HashSet<u16>,

    pub input: VecDeque<i32>,
    pub output: String,

    // The steps Back can undo, oldest first, and what the current one has changed so far.
    history: VecDeque<Undo>,
    changes: Vec<Change>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: Registers::new(),
            previous: Registers::new(),
            memory: Memory::new(),

            instruction_set: InstructionSet::default(),

            entry_point: 0,

            halted: false,
            waiting: false,

            executing: false,
            mc: 0,

            recording: None,

            breakpoints: HashSet::new(),

            input: VecDeque::new(),
            output: String::new(),

            history: VecDeque::new(),
            changes: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.entry_point = self.address(self.entry_point as i32);
        self.cpu.pc = self.entry_point;
        self.previous = self.cpu;

        self.halted = false;
        self.waiting = false;

        self.executing = false;
        self.mc = 0;

        self.input.clear();
        self.output.clear();
        self.history.clear();
    }

    pub fn clear(&mut self) {
        self.memory.reset();
        self.entry_point = 0;
        self.breakpoints.clear();

        self.reset();
    }

    // A cell edited by hand: the fixed cell can't be, and the steps before can't be undone.
    pub fn edit(&mut self, address: usize, word: i32) {
        if !self.is_fixed(address) {
            self.memory.write(address, word);
            self.history.clear();
        }
    }

    pub fn is_fixed(&self, address: usize) -> bool {
        self.instruction_set
            .format
            .fixed()
            .is_some_and(|(cell, _)| cell == address)
    }

    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    fn save(&self) -> Undo {
        Undo {
            cpu: self.cpu,
            previous: self.previous,
            halted: self.halted,
            waiting: self.waiting,
            executing: self.executing,
            mc: self.mc,
            output: self.output.len(),
            recorded: false,
            changes: Vec::new(),
        }
    }

    fn keep(&mut self, mut undo: Undo) {
        undo.changes = std::mem::take(&mut self.changes);

        // A step that changed nothing, like Step while IN waits, is not worth a Back.
        let same = (undo.cpu, undo.halted, undo.waiting, undo.executing, undo.mc, undo.output)
            == (self.cpu, self.halted, self.waiting, self.executing, self.mc, self.output.len());

        if same && undo.changes.is_empty() && !undo.recorded {
            return;
        }

        if self.history.len() == HISTORY_LIMIT {
            self.history.pop_front();
        }

        self.history.push_back(undo);
    }

    // Undoes the last step, micro step or micro step by hand; false if there is none.
    pub fn back(&mut self) -> bool {
        let Some(undo) = self.history.pop_back() else {
            return false;
        };

        for change in undo.changes.into_iter().rev() {
            match change {
                Change::Cell(address, word) => self.memory.write(address, word),
                Change::Input(value) => self.input.push_front(value),
                Change::Output(text) => self.output = text,
            }
        }

        // Drops what the step printed after the old end.
        self.output.truncate(undo.output);

        if undo.recorded
            && let Some(steps) = &mut self.recording
        {
            steps.pop();
        }

        self.cpu = undo.cpu;
        self.previous = undo.previous;
        self.halted = undo.halted;
        // With input sent since, IN no longer waits; Run would skip a machine that did.
        self.waiting = undo.waiting && self.input.is_empty();
        self.executing = undo.executing;
        self.mc = undo.mc;

        true
    }

    // Cleared by hand, the console stays clear through Back, past the limit's cuts too.
    pub fn clear_output(&mut self) {
        self.output.clear();

        for undo in &mut self.history {
            undo.changes.retain(|change| !matches!(change, Change::Output(_)));
        }
    }

    pub(super) fn pop_input(&mut self) -> Option<i32> {
        let value = self.input.pop_front()?;
        self.changes.push(Change::Input(value));

        Some(value)
    }

    pub fn push_input(&mut self, value: i32) {
        self.input.push_back(value);
        self.waiting = false;
    }

    // Not clamp(), which panics on a format whose minimum is above its maximum.
    pub fn set_acc(&mut self, value: i32) {
        let format = &self.instruction_set.format;

        self.cpu.acc = value.max(format.acc_min).min(format.acc_max);
    }

    pub(super) fn set_arithmetic_acc(&mut self, value: i64) {
        let format = self.instruction_set.format;
        let value = match format.data_encoding {
            // Past one end it comes round from the other, like SAP-1's A register.
            _ if format.flags => {
                let min = i64::from(format.acc_min);
                let span = (i64::from(format.acc_max) - min + 1).max(1);

                (min + (value - min).rem_euclid(span)) as i32
            }
            DataEncoding::TwosComplement => (value as i16) as i32,
            DataEncoding::Unsigned | DataEncoding::Lmc => {
                value.clamp(i32::MIN as i64, i32::MAX as i64) as i32
            }
        };

        self.set_acc(value);
    }

    // Plus and minus latch the flags, like SAP-1's flags register; loads leave them alone.
    pub(super) fn set_flags(&mut self, carry: bool) {
        if self.instruction_set.format.flags {
            self.cpu.carry = carry;
            self.cpu.zero = self.cpu.acc == 0;
        }
    }

    pub(super) fn address(&self, address: i32) -> u16 {
        let cells = self.instruction_set.format.cells().max(1) as i32;

        address.rem_euclid(cells) as u16
    }

    pub(super) fn normalize_pc(&mut self) {
        self.cpu.pc = self.address(self.cpu.pc as i32);
    }

    pub(super) fn set_data_bus(&mut self, value: i32) {
        self.cpu.mbr = match self.instruction_set.format.data_encoding {
            DataEncoding::Unsigned => value.clamp(0, 0xffff),
            DataEncoding::TwosComplement => value.clamp(i16::MIN as i32, i16::MAX as i32),
            DataEncoding::Lmc => value.clamp(-999, 999),
        };
    }

    // What the memory cell holds, unless the format fixes it.
    pub fn read(&self, address: usize) -> i32 {
        match self.instruction_set.format.fixed() {
            Some((cell, word)) if cell == address => word,
            _ => self.memory.read(address),
        }
    }

    pub(super) fn memory_to_data_bus(&mut self, value: i32) {
        let value = match self.instruction_set.format.data_encoding {
            DataEncoding::Unsigned => value.clamp(0, 0xffff),
            DataEncoding::TwosComplement => (value as u16 as i16) as i32,
            DataEncoding::Lmc => value.clamp(-999, 999),
        };

        self.set_data_bus(value);
    }

    pub(super) fn write_data_bus(&mut self) {
        let value = match self.instruction_set.format.data_encoding {
            DataEncoding::Unsigned => self.cpu.mbr,
            DataEncoding::TwosComplement => (self.cpu.mbr as i16 as u16) as i32,
            DataEncoding::Lmc => self.cpu.mbr.clamp(-999, 999),
        };

        let address = self.address(self.cpu.mar as i32);
        self.cpu.mar = address;
        self.changes
            .push(Change::Cell(address as usize, self.memory.read(address as usize)));
        self.memory.write(address as usize, value);
    }

    pub fn print(&mut self, text: &str) {
        self.output.push_str(text);

        if self.output.len() > OUTPUT_LIMIT {
            self.changes.push(Change::Output(self.output.clone()));
            self.output.drain(..OUTPUT_LIMIT / 2);
        }
    }

    // The micro steps the control unit is working through.
    pub fn steps(&self) -> &[MicroOp] {
        if self.executing {
            let (code, operand) = self.instruction_set.split(self.cpu.ir as i32);

            self.instruction_set.steps_for(code, operand)
        } else {
            &FETCH
        }
    }

    // Running off the end of a micro program ends the instruction.
    fn micro(&mut self) {
        let op = self
            .steps()
            .get(self.mc)
            .copied()
            .unwrap_or(MicroOp::McZero);

        self.waiting = false;

        execute(self, op);

        if !self.waiting && !matches!(op, MicroOp::InsToMc | MicroOp::McZero) {
            self.mc += 1;
        }
    }

    pub fn micro_step(&mut self) {
        if self.halted {
            return;
        }

        let undo = self.save();
        self.previous = self.cpu;
        self.micro();
        self.keep(undo);
    }

    // A micro step fired by hand; it does not move the control unit on.
    pub fn manual(&mut self, op: MicroOp) {
        let mut undo = self.save();
        self.previous = self.cpu;

        execute(self, op);

        if let Some(steps) = &mut self.recording {
            steps.push(op);
            undo.recorded = true;
        }

        self.keep(undo);
    }

    // Finishes the current instruction, from wherever the micro steps are.
    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        let undo = self.save();
        self.previous = self.cpu;

        let ended = (0..MICRO_LIMIT).any(|_| {
            self.micro();

            self.halted || self.waiting || (!self.executing && self.mc == 0)
        });

        if !ended {
            self.print("error: the instruction never ends\n");
            self.halted = true;
        }

        self.keep(undo);
    }

    // Returns true when it stopped on a breakpoint, or on an instruction that left the program
    // counter where it was if the format says so.
    pub fn run_steps(&mut self, steps: usize) -> bool {
        for _ in 0..steps {
            if self.halted || self.waiting {
                break;
            }

            let pc = self.cpu.pc;

            self.step();

            let stays = self.instruction_set.format.stop_when_pc_stays && self.cpu.pc == pc;

            // Checked after the step, so running from a breakpoint moves past it.
            if !self.halted && !self.waiting && (stays || self.breakpoints.contains(&self.cpu.pc)) {
                return true;
            }
        }

        false
    }

    // Runs to a halt, to `limit` instructions or to an IN with no input left. `done` sees every
    // instruction that finished, with the address and text it had before it ran.
    pub fn run(&mut self, limit: usize, mut done: impl FnMut(&mut Self, u16, &str)) -> Stop {
        for _ in 0..limit {
            let (pc, instruction) = (self.cpu.pc, self.current_instruction());
            let stopped = self.run_steps(1);

            if self.waiting {
                return Stop::NoInput;
            }

            done(self, pc, &instruction);

            if self.halted || stopped {
                return Stop::Halted;
            }
        }

        Stop::StepLimit
    }

    // The word the next fetch puts in the instruction register, from the cell pc->ab picks.
    pub fn next_word(&self) -> i32 {
        let cell = self.read(self.address(self.cpu.pc as i32) as usize);

        self.instruction_set.format.fetched(cell)
    }

    pub fn current_instruction(&self) -> String {
        self.describe(self.next_word())
    }

    pub fn describe(&self, raw: i32) -> String {
        let set = &self.instruction_set;
        let (code, operand) = set.split(raw);
        let operand = set.format.operand_text(operand);

        format!(
            "{}.{operand}  {} {operand}",
            set.format.code_text(code),
            set.mnemonic_for(code, set.split(raw).1)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::Program;

    fn word(opcode: i32, operand: i32) -> i32 {
        (opcode << 8) | operand
    }

    fn machine_with(program: &[i32]) -> Machine {
        let mut machine = Machine::new();
        machine.memory.load(program);
        machine
    }

    #[test]
    fn program_counter_wraps_instead_of_overflowing() {
        let mut machine = Machine::new();
        machine.cpu.pc = u16::MAX;

        machine.step();

        assert_eq!(machine.cpu.pc, 0);
    }

    #[test]
    fn program_counter_stays_inside_the_cells_of_its_mode() {
        let mut machine = Machine::new();
        machine.cpu.pc = 255;

        machine.step();

        assert_eq!(machine.cpu.pc, 0);

        machine.instruction_set = InstructionSet::johnny();
        machine.cpu.pc = 999;

        machine.step();

        assert_eq!(machine.cpu.pc, 999);
    }

    #[test]
    fn explicit_addresses_are_reduced_to_the_formats_cells() {
        let mut machine = machine_with(&[word(7, 50)]);
        machine.instruction_set.format.cells = Some(10);
        machine.memory.write(50, word(255, 0));

        machine.step();

        assert_eq!(machine.cpu.pc, 0);
        assert!(!machine.halted);
    }

    #[test]
    fn invalid_program_counters_are_reduced_before_fetching() {
        let mut machine = Machine::new();
        machine.instruction_set.format.cells = Some(10);
        machine.memory.write(5, word(255, 0));
        machine.cpu.pc = u16::MAX;

        machine.step();

        assert!(machine.halted);
        assert_eq!(machine.cpu.pc, 5);
    }

    #[test]
    fn accumulator_stays_clamped_on_multiply() {
        let mut program = vec![word(1, 10), word(5, 255), word(5, 255)];
        program.resize(11, 0);
        program[10] = 200;

        let mut machine = machine_with(&program);

        machine.run_steps(3);

        assert_eq!(machine.cpu.acc, ACC_MAX);
    }

    #[test]
    fn subtract_does_not_go_below_zero() {
        let mut machine = machine_with(&[word(4, 10)]);
        machine.memory.write(10, 10);
        machine.set_acc(5);

        machine.step();

        assert_eq!(machine.cpu.acc, 0);
    }

    #[test]
    fn store_and_load_move_through_the_buffer_register() {
        let mut machine = machine_with(&[word(1, 10), word(2, 100), word(1, 100), word(255, 0)]);
        machine.memory.write(10, 42);

        machine.run_steps(3);

        assert_eq!(machine.memory.read(100), 42);
        assert_eq!(machine.cpu.mbr, 42);
        assert_eq!(machine.cpu.mar, 100);
        assert_eq!(machine.cpu.acc, 42);

        machine.step();

        assert!(machine.halted);
    }

    #[test]
    fn division_by_zero_halts_with_a_message() {
        let mut machine = machine_with(&[word(6, 0)]);

        machine.step();

        assert!(machine.halted);
        assert!(machine.output.contains("division by zero"));
    }

    #[test]
    fn run_steps_stops_at_halt() {
        let mut machine = machine_with(&[word(1, 10), word(255, 0), word(3, 10)]);
        machine.memory.write(10, 1);

        machine.run_steps(100);

        assert!(machine.halted);
        assert_eq!(machine.cpu.acc, 1);
        assert_eq!(machine.cpu.pc, 1);
    }

    #[test]
    fn reset_keeps_memory_and_returns_to_entry_point() {
        let mut machine = machine_with(&[word(1, 10), word(255, 0)]);
        machine.memory.write(10, 5);
        machine.entry_point = 0;

        machine.run_steps(8);
        machine.reset();

        assert!(!machine.halted);
        assert_eq!(machine.cpu.pc, 0);
        assert_eq!(machine.cpu.acc, 0);
        assert_eq!(machine.memory.read(0), word(1, 10));
    }

    #[test]
    fn countdown_asset_runs_to_completion() {
        let raw = std::fs::read_to_string("assets/countdown.json").unwrap();
        let program: Program = serde_json::from_str(&raw).unwrap();

        let mut machine = Machine::new();
        program.apply(&mut machine);
        machine.run_steps(1000);

        assert!(machine.halted);
        assert_eq!(machine.output, "5\n4\n3\n2\n1\n");
    }

    #[test]
    fn out_of_range_memory_access_is_ignored() {
        let mut machine = Machine::new();

        machine.memory.write(MEMORY_SIZE_FOR_TESTS, 7);

        assert_eq!(machine.memory.read(MEMORY_SIZE_FOR_TESTS), 0);
    }

    #[test]
    fn add_and_subtract_use_values_from_memory() {
        let mut machine = machine_with(&[word(1, 10), word(3, 11), word(4, 10)]);
        machine.memory.write(10, 7);
        machine.memory.write(11, 2);

        machine.run_steps(3);

        assert_eq!(machine.cpu.acc, 2);
        assert_eq!(machine.cpu.mar, 10);
        assert_eq!(machine.cpu.mbr, 7);
    }

    #[test]
    fn inc_and_dec_modify_memory_with_clamping() {
        let mut machine = machine_with(&[word(11, 10), word(12, 11)]);
        machine.memory.write(10, ACC_MAX);
        machine.memory.write(11, 0);

        machine.run_steps(2);

        assert_eq!(machine.memory.read(10), ACC_MAX);
        assert_eq!(machine.memory.read(11), 0);
    }

    #[test]
    fn null_sets_memory_to_zero() {
        let mut machine = machine_with(&[word(13, 10)]);
        machine.memory.write(10, 123);

        machine.step();

        assert_eq!(machine.memory.read(10), 0);
    }

    #[test]
    fn tst_skips_the_next_instruction_when_memory_is_zero() {
        let mut machine = machine_with(&[word(14, 10), word(3, 11), word(255, 0)]);
        machine.memory.write(11, 7);

        machine.run_steps(2);

        assert_eq!(machine.cpu.acc, 0);
        assert!(machine.halted);
        assert_eq!(machine.cpu.pc, 2);
    }

    #[test]
    fn input_waits_then_resumes_when_a_value_arrives() {
        let mut machine = machine_with(&[word(9, 0), word(10, 0), word(255, 0)]);

        machine.run_steps(10);

        assert!(machine.waiting);
        assert_eq!(machine.cpu.pc, 0);

        machine.push_input(7);
        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.output, "7\n");
    }

    #[test]
    fn output_is_capped() {
        let mut machine = machine_with(&[word(10, 0), word(7, 0)]);

        machine.run_steps(OUTPUT_LIMIT * 2);

        assert!(machine.output.len() <= OUTPUT_LIMIT);
    }

    #[test]
    fn run_stops_on_a_breakpoint_and_continues_past_it() {
        let mut machine = Machine::new();
        machine.breakpoints.insert(2);

        assert!(machine.run_steps(10));
        assert_eq!(machine.cpu.pc, 2);

        assert!(!machine.run_steps(10));
        assert_eq!(machine.cpu.pc, 12);
    }

    #[test]
    fn saved_program_carries_its_instruction_set() {
        let mut machine = machine_with(&[word(1, 10), word(255, 0)]);
        machine.instruction_set.name = "Custom".to_string();
        machine.instruction_set.definitions[1].code = 77;
        machine.breakpoints.insert(1);

        let path = crate::utilities::temp_path("roundtrip.json");
        Program::from_machine("roundtrip".to_string(), &machine)
            .save(&path)
            .unwrap();
        let program = Program::load(&path, &machine.instruction_set).unwrap();
        std::fs::remove_file(&path).unwrap();

        let mut loaded = Machine::new();
        loaded.breakpoints.insert(5);
        program.apply(&mut loaded);

        assert_eq!(program.name, "roundtrip");
        assert_eq!(loaded.memory.read(0), word(1, 10));
        assert_eq!(loaded.instruction_set.name, "Custom");
        assert_eq!(loaded.instruction_set.definitions[1].code, 77);
        assert!(loaded.breakpoints.is_empty());
    }

    #[test]
    fn micro_steps_walk_through_fetch_and_execute() {
        let mut machine = machine_with(&[word(1, 10)]);
        machine.memory.write(10, 42);

        for _ in 0..FETCH.len() {
            machine.micro_step();
        }

        assert!(machine.executing);
        assert_eq!(machine.cpu.ir as i32, word(1, 10));
        assert_eq!(machine.cpu.acc, 0);

        machine.step();

        assert!(!machine.executing);
        assert_eq!(machine.cpu.acc, 42);
        assert_eq!(machine.cpu.pc, 1);
    }

    #[test]
    fn recorded_micro_steps_run_as_an_instruction() {
        let mut machine = machine_with(&[word(77, 10)]);
        machine.memory.write(10, 5);
        machine.recording = Some(Vec::new());

        for op in [
            MicroOp::InsToAb,
            MicroOp::RamToDb,
            MicroOp::DbToAcc,
            MicroOp::Plus,
            MicroOp::PcInc,
        ] {
            machine.manual(op);
        }

        let micro = machine.recording.take().unwrap();
        machine.instruction_set.definitions.push(Definition {
            code: 77,
            operand: None,
            operand_mask: None,
            mnemonic: "DBL".to_string(),
            opcode: Opcode::Custom,
            micro,
        });

        machine.reset();
        machine.step();

        assert_eq!(machine.cpu.acc, 10);
        assert_eq!(machine.cpu.pc, 1);
    }

    #[test]
    fn micro_program_that_never_ends_halts() {
        let mut machine = machine_with(&[word(77, 0)]);
        machine.instruction_set.definitions.push(Definition {
            code: 77,
            opcode: Opcode::Custom,
            micro: vec![MicroOp::InsToMc],
            ..Default::default()
        });

        machine.step();

        assert!(machine.halted);
    }

    #[test]
    fn instruction_sets_saved_before_micro_steps_still_load() {
        let old = r#"{"name":"Old","definitions":[{"code":1,"mnemonic":"LOAD","opcode":"Load"}]}"#;
        let set: InstructionSet = serde_json::from_str(old).unwrap();

        assert_eq!(set.steps_for(1, 0), Opcode::Load.micro());
    }

    #[test]
    fn johnny_program_file_runs_with_decimal_words() {
        let program = Program::from_ram("sum", include_str!("../../assets/sum.ram")).unwrap();

        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        program.apply(&mut machine);
        machine.run_steps(100);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(12), 12);
        assert_eq!(machine.describe(4012), "04.012  SAVE 012");
        assert!(
            program
                .to_ram()
                .starts_with("1010\r\n2011\r\n4012\r\n10000\r\n")
        );
        assert_eq!(program.to_ram().lines().count(), 1000);
    }

    #[test]
    fn johnny_microcode_file_becomes_an_instruction_set() {
        let mut lines = vec!["0".to_string(); 200];

        for (line, id) in [(0, 8), (1, 2), (2, 3), (3, 5)] {
            lines[line] = id.to_string();
        }

        for (line, id) in [(10, 4), (11, 2), (12, 18), (13, 9), (14, 7)] {
            lines[line] = id.to_string();
        }

        lines.extend(["FETCH".to_string(), "TAKE".to_string()]);

        let set = InstructionSet::from_mc("mine", &lines.join("\r\n")).unwrap();

        assert_eq!(set.format, Format::johnny());
        assert_eq!(set.mnemonic_for(1, 0), "TAKE");
        assert_eq!(set.steps_for(1, 0), Opcode::Load.micro());
        assert!(set.find(2).is_none());
    }

    #[test]
    fn a_custom_format_runs_its_own_words() {
        // Octal words: one digit of code, then three of operand.
        let set: InstructionSet = serde_json::from_str(
            r#"{"name":"Octal","format":{"radix":8,"code_digits":1,"operand_digits":3},
                "definitions":[{"code":1,"mnemonic":"LOAD","opcode":"Load"},
                {"code":2,"mnemonic":"ADD","opcode":"Add"},
                {"code":3,"mnemonic":"STORE","opcode":"Store"},
                {"code":4,"mnemonic":"OUT","opcode":"Output"},
                {"code":7,"mnemonic":"HALT","opcode":"Halt"}]}"#,
        )
        .unwrap();
        let octal = |code: i32, operand: i32| code * 0o1000 + operand;

        set.check().unwrap();

        let mut machine = machine_with(&[
            octal(1, 0o100),
            octal(2, 0o101),
            octal(3, 0o102),
            octal(4, 0),
            octal(7, 0),
        ]);
        machine.instruction_set = set;
        machine.memory.write(0o100, 5);
        machine.memory.write(0o101, 7);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(0o102), 12);
        assert_eq!(machine.output, "12\n");
        assert_eq!(machine.instruction_set.limits(), (7, 0o777, 0o7777));
        assert_eq!(machine.instruction_set.format.cells(), 512);
        assert_eq!(machine.describe(octal(3, 0o102)), "3.102  STORE 102");
    }

    #[test]
    fn a_signed_accumulator_clamps_at_its_negative_minimum() {
        let mut machine = machine_with(&[word(4, 10), word(4, 10), word(10, 0), word(255, 0)]);
        machine.instruction_set.format.acc_min = -10;
        machine.memory.write(10, 7);

        machine.step();

        assert_eq!(machine.cpu.acc, -7);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.cpu.acc, -10);
        assert_eq!(machine.output, "-10\n");
    }

    #[test]
    fn past_the_last_cell_the_program_counter_does_what_the_format_says() {
        for (overflow, pc, halted) in [
            (PcOverflow::Wrap, 0, false),
            (PcOverflow::Stay, 15, false),
            (PcOverflow::Halt, 15, true),
        ] {
            let mut machine = Machine::new();
            machine.instruction_set.format.cells = Some(16);
            machine.instruction_set.format.pc_overflow = overflow;
            machine.cpu.pc = 15;

            machine.step();

            assert_eq!((machine.cpu.pc, machine.halted), (pc, halted), "{overflow:?}");
        }
    }

    #[test]
    fn an_unknown_code_is_skipped_or_halts() {
        for (unknown, pc, halted) in [(UnknownCode::Nop, 1, false), (UnknownCode::Halt, 0, true)] {
            let mut machine = machine_with(&[word(200, 0)]);
            machine.instruction_set.format.unknown_code = unknown;

            machine.step();

            assert_eq!((machine.cpu.pc, machine.halted), (pc, halted), "{unknown:?}");
        }
    }

    #[test]
    fn run_stops_on_an_instruction_that_keeps_the_program_counter_if_the_format_says_so() {
        let mut machine = machine_with(&[word(7, 0)]);

        assert!(!machine.run_steps(5));

        machine.instruction_set.format.stop_when_pc_stays = true;

        assert!(machine.run_steps(5));
        assert!(!machine.halted);
        assert_eq!(machine.cpu.pc, 0);
    }

    #[test]
    fn every_bundled_example_runs_with_its_preset() {
        for example in crate::program::examples() {
            let mut machine = Machine::new();
            assert_eq!(
                example.program.instruction_set.as_ref().unwrap().name,
                example.preset.name,
                "{} did not carry its instruction set",
                example.name
            );
            example.program.apply(&mut machine);

            if example.name == "LMC echo" || example.name == "MARIE echo" {
                machine.push_input(42);
            }

            machine.run_steps(100);

            assert!(machine.halted, "{} did not halt", example.name);
            let expected = match example.name {
                "Standard countdown" => "5\n4\n3\n2\n1\n",
                "LMC echo" | "MARIE echo" => "42\n",
                "CARDIAC digits" => "4\n7\n2\n",
                "SAP-1 count" => "0\n51\n102\n153\n204\n255\n255\n204\n153\n102\n51\n0\n",
                "Johnny sum" | "Bonsai increment" => "",
                name => panic!("unexpected example {name}"),
            };

            assert_eq!(machine.output, expected, "{}", example.name);

            match example.name {
                "Johnny sum" => assert_eq!(machine.memory.read(12), 12),
                "Bonsai increment" => assert_eq!(machine.memory.read(5), 5),
                _ => {}
            }
        }
    }

    #[test]
    fn lmc_branches_on_zero_and_nonnegative_accumulator() {
        for (word, acc, expected_pc) in [
            (704, 0, 4),
            (704, 1, 1),
            (804, -1, 1),
            (804, 0, 4),
            (804, 1, 4),
        ] {
            let mut machine = machine_with(&[word]);
            machine.instruction_set = InstructionSet::lmc();
            machine.set_acc(acc);

            machine.step();

            assert_eq!(machine.cpu.pc, expected_pc, "word {word}, AC {acc}");
        }
    }

    #[test]
    fn lmc_input_and_output_use_the_fixed_instruction_operands() {
        let mut machine = machine_with(&[901, 902, 0]);
        machine.instruction_set = InstructionSet::lmc();
        machine.push_input(321);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.output, "321\n");

        let mut invalid = machine_with(&[903]);
        invalid.instruction_set = InstructionSet::lmc();
        invalid.step();

        assert!(!invalid.halted);
        assert_eq!(invalid.cpu.pc, 1);
    }

    #[test]
    fn lmc_stores_and_loads_signed_values() {
        let mut machine = machine_with(&[310, 510, 902, 0]);
        machine.instruction_set = InstructionSet::lmc();
        machine.set_acc(-1);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(10), -1);
        assert_eq!(machine.cpu.acc, -1);
        assert_eq!(machine.output, "-1\n");
    }

    #[test]
    fn lmc_addition_uses_a_negative_value_loaded_from_memory() {
        let mut machine = machine_with(&[510, 211, 312, 512, 111, 902, 0]);
        machine.instruction_set = InstructionSet::lmc();
        machine.memory.write(10, 0);
        machine.memory.write(11, 1);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(12), -1);
        assert_eq!(machine.cpu.acc, 0);
        assert_eq!(machine.output, "0\n");
    }

    #[test]
    fn lmc_brp_treats_a_reloaded_negative_value_as_negative() {
        let mut machine = machine_with(&[510, 211, 312, 512, 807]);
        machine.instruction_set = InstructionSet::lmc();
        machine.memory.write(10, 0);
        machine.memory.write(11, 1);

        machine.run_steps(4);
        machine.step();

        assert_eq!(machine.memory.read(12), -1);
        assert_eq!(machine.cpu.acc, -1);
        assert_eq!(machine.cpu.pc, 5);
    }

    #[test]
    fn marie_skipcond_checks_less_zero_equal_zero_and_greater_zero() {
        for (acc, condition, should_skip) in [
            (-1, 0x000, true),
            (0, 0x000, false),
            (0, 0x400, true),
            (1, 0x400, false),
            (1, 0x800, true),
            (0, 0x800, false),
        ] {
            let mut machine = machine_with(&[0x8000 + condition]);
            machine.instruction_set = InstructionSet::marie();
            machine.set_acc(acc);

            machine.step();

            assert_eq!(machine.cpu.pc, if should_skip { 2 } else { 1 });
        }
    }

    #[test]
    fn marie_skipcond_ignores_the_low_ten_operand_bits() {
        let mut machine = machine_with(&[0x8401]);
        machine.instruction_set = InstructionSet::marie();

        machine.step();

        assert_eq!(machine.cpu.pc, 2);
        assert_eq!(machine.instruction_set.mnemonic_for(8, 0x401), "Skipcond =0");
    }

    #[test]
    fn marie_indirect_addresses_are_reduced_to_twelve_bits_and_cells() {
        let mut machine = machine_with(&[0xe010]);
        machine.instruction_set = InstructionSet::marie();
        machine.memory.write(0x010, 0x1014);
        machine.set_acc(42);

        machine.step();

        assert_eq!(machine.memory.read(0x014), 42);
        assert_eq!(machine.memory.read(0x1014), 0);

        let mut jump = machine_with(&[0xc010]);
        jump.instruction_set = InstructionSet::marie();
        jump.memory.write(0x010, 0x1014);

        jump.step();

        assert_eq!(jump.cpu.pc, 0x014);
    }

    #[test]
    fn marie_negative_indirect_jump_addresses_wrap_euclideanly() {
        let mut machine = machine_with(&[0xc010]);
        machine.instruction_set = InstructionSet::marie();
        machine.memory.write(0x010, 0xffff);

        machine.step();

        assert_eq!(machine.cpu.pc, 0xfff);
    }

    #[test]
    fn marie_jns_and_jumpi_return_to_the_instruction_after_the_call() {
        let mut memory = vec![0; 12];
        memory[0] = 0x000a;
        memory[1] = 0x6000;
        memory[2] = 0x7000;
        memory[11] = 0xc00a;

        let mut machine = machine_with(&memory);
        machine.instruction_set = InstructionSet::marie();
        machine.set_acc(42);
        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(10), 1);
        assert_eq!(machine.output, "42\n");
    }

    #[test]
    fn marie_indirect_add_load_and_store_follow_the_pointer_cell() {
        let mut machine = machine_with(&[0xd00a, 0xb00a, 0xe00a, 0x1014, 0x6000, 0x7000]);
        machine.instruction_set = InstructionSet::marie();
        machine.memory.write(10, 20);
        machine.memory.write(20, 5);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(20), 10);
        assert_eq!(machine.cpu.acc, 10);
        assert_eq!(machine.output, "10\n");
    }

    #[test]
    fn marie_stores_and_loads_negative_accumulator_values_in_twos_complement() {
        let mut machine = machine_with(&[0xa000, 0x400a, 0x200b, 0x100b, 0x6000, 0x7000]);
        machine.instruction_set = InstructionSet::marie();
        machine.memory.write(10, 123);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(11), 65_413);
        assert_eq!(machine.cpu.acc, -123);
        assert_eq!(machine.output, "-123\n");
    }

    #[test]
    fn marie_addition_wraps_at_the_signed_word_boundary() {
        let mut machine = machine_with(&[0x100a, 0x300b]);
        machine.instruction_set = InstructionSet::marie();
        machine.memory.write(10, 0x7fff);
        machine.memory.write(11, 1);

        machine.run_steps(2);

        assert_eq!(machine.cpu.acc, i16::MIN as i32);
    }

    #[test]
    fn marie_subtraction_wraps_at_the_signed_word_boundary() {
        let mut machine = machine_with(&[0x100a, 0x400b]);
        machine.instruction_set = InstructionSet::marie();
        machine.memory.write(10, 0x8000);
        machine.memory.write(11, 1);

        machine.run_steps(2);

        assert_eq!(machine.cpu.acc, i16::MAX as i32);
    }

    #[test]
    fn the_fetched_word_of_a_cell_is_what_the_fetch_leaves_in_the_instruction_register() {
        for preset in InstructionSet::presets() {
            for cell in [0, 5, 902, -1, -999, 1500, 0x8400, 0xffff, 70_000] {
                // Cell 1, since CARDIAC's cell 0 is fixed.
                let mut machine = machine_with(&[0, cell]);
                machine.instruction_set = preset.clone();
                machine.cpu.pc = 1;

                for op in FETCH {
                    machine.manual(op);
                }

                assert_eq!(
                    machine.cpu.ir as i32,
                    preset.format.fetched(cell),
                    "{} {cell}",
                    preset.name
                );
            }
        }
    }

    // CARDIAC's cell 00 is its loader, so a program starts at 01.
    fn cardiac(program: &[i32]) -> Machine {
        let mut machine = machine_with(&[&[0], program].concat());
        machine.instruction_set = InstructionSet::cardiac();
        machine.entry_point = 1;
        machine.reset();
        machine
    }

    #[test]
    fn every_cardiac_instruction_does_what_its_card_says() {
        let mut machine = cardiac(&[20, 120, 221, 722, 307, 900, 623, 523, 410, 830]);
        machine.memory.write(21, 5);
        machine.memory.write(22, 20);
        machine.memory.write(30, 931);
        machine.set_acc(3);
        machine.push_input(7);

        let step = |machine: &mut Machine| {
            machine.step();
            (machine.cpu.acc, machine.cpu.pc)
        };

        assert_eq!(step(&mut machine), (3, 2), "INP 20 leaves the accumulator alone");
        assert_eq!(machine.memory.read(20), 7);
        assert_eq!(step(&mut machine), (7, 3), "CLA 20");
        assert_eq!(step(&mut machine), (12, 4), "ADD 21");
        assert_eq!(step(&mut machine), (-8, 5), "SUB 22");
        assert_eq!(step(&mut machine), (-8, 7), "TAC 07 jumps on a negative accumulator");
        assert_eq!(step(&mut machine), (-8, 8), "STO 23");
        assert_eq!(machine.memory.read(23), -8);
        assert_eq!(step(&mut machine), (-8, 9), "OUT 23");
        assert_eq!(machine.output, "-8\n");
        assert_eq!(step(&mut machine), (-80, 10), "SFT 10");
        assert_eq!(step(&mut machine), (-80, 30), "JMP 30");
        assert_eq!(machine.memory.read(99), 811);
        assert_eq!(step(&mut machine), (-80, 31), "HRS 31");
        assert!(machine.halted);

        for acc in [0, 1] {
            let mut machine = cardiac(&[307]);
            machine.set_acc(acc);
            machine.step();

            assert_eq!(machine.cpu.pc, 2, "TAC on {acc}");
        }
    }

    #[test]
    fn cardiac_jmp_leaves_a_jump_back_that_returns() {
        // 01 JMP 10, 02 HRS 00; 10 CLA 99, 11 STO 12, 12 the jump back.
        let mut machine = cardiac(&[810, 900]);
        machine.memory.write(10, 199);
        machine.memory.write(11, 612);

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!(machine.memory.read(12), 802);
        assert_eq!(machine.cpu.pc, 0);
    }

    #[test]
    fn cardiac_sft_drops_the_digits_it_shifts_out_and_keeps_the_sign() {
        for (acc, word, expected) in [
            (1234, 400, 1234),
            (1234, 410, 2340),
            (1234, 401, 123),
            (1234, 422, 34),
            (1234, 413, 2),
            (-1234, 411, -234),
            (1234, 440, 0),
            (1234, 450, 0),
            (1234, 405, 0),
            (-9999, 499, 0),
        ] {
            let mut machine = cardiac(&[word]);
            machine.set_acc(acc);
            machine.step();

            assert_eq!(machine.cpu.acc, expected, "{acc} {word}");
        }
    }

    #[test]
    fn cardiac_sto_keeps_the_low_three_digits_and_the_sign() {
        for (acc, stored) in [(1234, 234), (-1234, -234), (9999, 999), (-5, -5), (1000, 0)] {
            let mut machine = cardiac(&[610]);
            machine.set_acc(acc);
            machine.step();

            assert_eq!(machine.memory.read(10), stored, "{acc}");
        }
    }

    #[test]
    fn cardiac_inp_clamps_input_to_the_word_range() {
        for (input, stored) in [(1000, 999), (2500, 999), (-1000, -999), (42, 42)] {
            let mut machine = cardiac(&[10]);
            machine.push_input(input);
            machine.step();

            assert_eq!(machine.memory.read(10), stored, "{input}");
        }
    }

    #[test]
    fn cardiac_cell_00_always_holds_its_loader() {
        // STO 00, then a write by hand.
        let mut machine = cardiac(&[600, 100]);
        machine.set_acc(42);
        machine.run_steps(2);

        assert_eq!((machine.read(0), machine.cpu.acc), (1, 1));

        machine.memory.write(0, 5);
        machine.cpu.pc = 0;

        assert_eq!(machine.read(0), 1);
        assert_eq!(machine.current_instruction(), "0.01  INP 01");

        // From 00 the loader reads the next input into 01 and runs it: HRS 00 here.
        let mut boot = cardiac(&[]);
        boot.entry_point = 0;
        boot.reset();
        boot.push_input(900);
        boot.run_steps(10);

        assert!(boot.halted);
        assert_eq!((boot.memory.read(1), boot.cpu.pc), (900, 0));

        // Other machines have no fixed cell.
        let mut lmc = machine_with(&[5]);
        lmc.instruction_set = InstructionSet::lmc();

        assert_eq!(lmc.read(0), 5);
    }

    fn sap1(program: &[i32]) -> Machine {
        let mut machine = machine_with(program);
        machine.instruction_set = InstructionSet::sap1();
        machine
    }

    #[test]
    fn every_sap1_instruction_does_what_its_table_says() {
        let mut machine = sap1(&[
            0x00, 0x1e, 0x2f, 0x75, 0xf0, 0x3d, 0x88, 0xf0, 0x59, 0x4c, 0xe0, 0x64, 0, 44, 200, 100,
        ]);

        let step = |machine: &mut Machine| {
            machine.step();
            (machine.cpu.acc, machine.cpu.pc, machine.cpu.carry, machine.cpu.zero)
        };

        assert_eq!(step(&mut machine), (0, 1, false, false), "NOP");
        assert_eq!(step(&mut machine), (200, 2, false, false), "LDA E");
        assert_eq!(step(&mut machine), (44, 3, true, false), "ADD F");
        assert_eq!(step(&mut machine), (44, 5, true, false), "JC 5");
        assert_eq!(step(&mut machine), (0, 6, true, true), "SUB D");
        assert_eq!(step(&mut machine), (0, 8, true, true), "JZ 8");
        assert_eq!(step(&mut machine), (9, 9, true, true), "LDI 9 leaves the flags alone");
        assert_eq!(step(&mut machine), (9, 10, true, true), "STA C");
        assert_eq!(machine.memory.read(0xc), 9);
        assert_eq!(step(&mut machine), (9, 11, true, true), "OUT");
        assert_eq!(machine.output, "9\n");
        assert_eq!(step(&mut machine), (9, 4, true, true), "JMP 4");
        assert_eq!(step(&mut machine), (9, 4, true, true), "HLT");
        assert!(machine.halted);
        assert_eq!(machine.describe(0x75), "7.5  JC 5");
    }

    #[test]
    fn sap1_add_and_sub_set_the_flags_that_jc_and_jz_test() {
        for (a, op, b, acc, carry, zero) in [
            (200, "ADD", 100, 44, true, false),
            (200, "ADD", 56, 0, true, true),
            (1, "ADD", 2, 3, false, false),
            (5, "SUB", 3, 2, true, false),
            (5, "SUB", 5, 0, true, true),
            (3, "SUB", 5, 254, false, false),
        ] {
            let code = if op == "ADD" { 0x2f } else { 0x3f };

            for (jump, taken) in [(0x79, carry), (0x89, zero)] {
                // LDA E, ADD or SUB F, then JC or JZ 9.
                let mut machine = sap1(&[0x1e, code, jump]);
                machine.memory.write(0xe, a);
                machine.memory.write(0xf, b);

                machine.run_steps(3);

                let flags = (machine.cpu.acc, machine.cpu.carry, machine.cpu.zero);
                assert_eq!(flags, (acc, carry, zero), "{a} {op} {b}");
                assert_eq!(machine.cpu.pc, if taken { 9 } else { 3 }, "{a} {op} {b} {jump:x}");
            }
        }
    }

    #[test]
    fn sap1_a_register_wraps_where_other_accumulators_stop() {
        let mut machine = sap1(&[]);
        machine.set_acc(255);

        machine.manual(MicroOp::AccInc);
        assert_eq!(machine.cpu.acc, 0);

        machine.manual(MicroOp::AccDec);
        assert_eq!(machine.cpu.acc, 255);

        machine.cpu.mbr = 0x200 + 3;
        machine.manual(MicroOp::Plus);
        assert_eq!((machine.cpu.acc, machine.cpu.carry), (2, true));

        // A machine without flags still stops at its ends and sets none.
        let mut standard = Machine::new();
        standard.set_acc(ACC_MAX);
        standard.cpu.mbr = 1;
        standard.manual(MicroOp::Plus);

        assert_eq!(standard.cpu.acc, ACC_MAX);
        assert!(!standard.cpu.carry && !standard.cpu.zero);

        // Reset clears the flags.
        machine.reset();
        assert!(!machine.cpu.carry && !machine.cpu.zero);
    }

    #[test]
    fn the_next_word_is_read_from_the_cell_the_fetch_reduces_the_pc_to() {
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::lmc();
        machine.memory.write(5, 901);
        machine.memory.write(105, 902);
        machine.cpu.pc = 105;

        let next = machine.next_word();

        for op in FETCH {
            machine.manual(op);
        }

        assert_eq!((next, machine.cpu.ir as i32), (901, 901));
    }

    #[test]
    fn back_undoes_every_step_down_to_the_start() {
        // IN, STORE 20, OUT, HALT.
        let mut machine = machine_with(&[word(9, 0), word(2, 20), word(10, 0), word(255, 0)]);
        machine.memory.write(20, 3);
        machine.push_input(7);
        machine.push_input(8);
        let start = machine.cpu;

        machine.run_steps(10);

        assert!(machine.halted);
        assert_eq!((machine.memory.read(20), machine.output.as_str()), (7, "7\n"));

        machine.back();

        assert!(!machine.halted && machine.cpu.pc == 3);

        while machine.back() {}

        assert!(machine.cpu == start && machine.previous == start);
        assert_eq!((machine.memory.read(20), machine.output.as_str()), (3, ""));
        assert_eq!(machine.input, [7, 8]);
        assert!(!machine.can_go_back());
    }

    #[test]
    fn back_undoes_micro_steps_and_steps_by_hand_one_at_a_time() {
        let mut machine = machine_with(&[word(1, 10)]);
        machine.recording = Some(Vec::new());

        machine.micro_step();
        machine.micro_step();
        machine.manual(MicroOp::AccInc);

        machine.back();

        assert_eq!((machine.cpu.acc, machine.recording.as_deref()), (0, Some(&[][..])));

        machine.back();

        assert_eq!((machine.mc, machine.executing, machine.cpu.mar), (1, false, 0));

        // Step while IN waits changes nothing, so there is nothing to undo for it.
        let mut waiting = machine_with(&[word(9, 0)]);
        waiting.step();
        waiting.step();

        assert!(waiting.back() && !waiting.back());
    }

    #[test]
    fn back_brings_back_output_the_limit_cut_off() {
        let mut machine = machine_with(&[word(10, 0), word(7, 0)]);

        machine.run_steps(OUTPUT_LIMIT - 4);

        let output = machine.output.clone();
        machine.run_steps(20);

        assert!(machine.output.len() < output.len());

        for _ in 0..20 {
            machine.back();
        }

        assert_eq!(machine.output, output);
    }

    #[test]
    fn back_brings_back_no_output_cleared_by_hand() {
        let mut machine = machine_with(&[word(10, 0), word(7, 0)]);

        machine.run_steps(OUTPUT_LIMIT + 100);
        machine.clear_output();

        while machine.back() {}

        assert_eq!(machine.output, "");
    }

    // A step by hand while IN waited, then input sent: back there, the run reads the input.
    #[test]
    fn back_to_a_wait_with_input_sent_since_does_not_wait() {
        let mut machine = machine_with(&[word(9, 0)]);

        machine.step();
        machine.manual(MicroOp::AccInc);
        machine.push_input(5);
        machine.step();
        machine.back();
        machine.back();

        assert!(!machine.waiting && machine.input == [5]);

        machine.run_steps(1);

        assert_eq!(machine.cpu.acc, 5);
    }

    #[test]
    fn history_is_bounded_and_forgotten_on_reset_and_on_edits() {
        let mut machine = Machine::new();
        machine.run_steps(HISTORY_LIMIT + 100);

        let mut undone = 0;

        while machine.back() {
            undone += 1;
        }

        assert_eq!(undone, HISTORY_LIMIT);

        machine.step();
        machine.edit(5, 1);

        assert!(!machine.can_go_back() && machine.memory.read(5) == 1);

        machine.step();
        machine.reset();

        assert!(!machine.can_go_back());

        // The fixed cell takes no edit at all.
        let mut cardiac = cardiac(&[]);
        cardiac.step();
        cardiac.edit(0, 5);

        assert_eq!(cardiac.memory.read(0), 0);
        assert!(cardiac.can_go_back());
    }

    const MEMORY_SIZE_FOR_TESTS: usize = crate::machine::MEMORY_SIZE + 1;
}
