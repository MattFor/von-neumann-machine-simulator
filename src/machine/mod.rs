mod executor;
mod instruction_decoder;
mod instructions;
mod memory;
mod registers;

use std::collections::{HashSet, VecDeque};

pub use instruction_decoder::encode;
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
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.pc = self.entry_point;
        self.previous = self.cpu;

        self.halted = false;
        self.waiting = false;

        self.executing = false;
        self.mc = 0;

        self.input.clear();
        self.output.clear();
    }

    pub fn clear(&mut self) {
        self.memory.reset();
        self.entry_point = 0;
        self.breakpoints.clear();

        self.reset();
    }

    pub fn push_input(&mut self, value: i32) {
        self.input.push_back(value);
        self.waiting = false;
    }

    pub fn print(&mut self, text: &str) {
        self.output.push_str(text);

        if self.output.len() > OUTPUT_LIMIT {
            self.output.drain(..OUTPUT_LIMIT / 2);
        }
    }

    // The micro steps the control unit is working through.
    pub fn steps(&self) -> &[MicroOp] {
        if self.executing {
            self.instruction_set.steps((self.cpu.ir >> 8) as i32)
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

        self.previous = self.cpu;
        self.micro();
    }

    // A micro step fired by hand; it does not move the control unit on.
    pub fn manual(&mut self, op: MicroOp) {
        self.previous = self.cpu;

        execute(self, op);

        if let Some(steps) = &mut self.recording {
            steps.push(op);
        }
    }

    // Finishes the current instruction, from wherever the micro steps are.
    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        self.previous = self.cpu;

        for _ in 0..MICRO_LIMIT {
            self.micro();

            if self.halted || self.waiting || (!self.executing && self.mc == 0) {
                return;
            }
        }

        self.print("error: the instruction never ends\n");
        self.halted = true;
    }

    // Returns true when it stopped on a breakpoint.
    pub fn run_steps(&mut self, steps: usize) -> bool {
        for _ in 0..steps {
            if self.halted || self.waiting {
                break;
            }

            self.step();

            // Checked after the step, so running from a breakpoint moves past it.
            if !self.halted && !self.waiting && self.breakpoints.contains(&self.cpu.pc) {
                return true;
            }
        }

        false
    }

    pub fn current_instruction(&self) -> String {
        self.describe(self.memory.read(self.cpu.pc as usize))
    }

    pub fn describe(&self, raw: i32) -> String {
        format!(
            "{:04X}  {} {:02X}",
            raw as u16,
            self.instruction_set.mnemonic((raw >> 8) & 0xff),
            raw & 0xff
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
        machine.cpu.set_acc(5);

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

        let path = std::env::temp_dir().join("vnm-roundtrip-test.json");
        Program::from_machine("roundtrip".to_string(), &machine)
            .save(&path)
            .unwrap();
        let program = Program::load(&path).unwrap();
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

        assert_eq!(set.steps(1), Opcode::Load.micro());
    }

    const MEMORY_SIZE_FOR_TESTS: usize = crate::machine::MEMORY_SIZE + 1;
}
