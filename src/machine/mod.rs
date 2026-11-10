mod executor;
mod instruction_decoder;
mod instructions;
mod memory;
mod registers;

use std::collections::{HashSet, VecDeque};

pub use executor::execute;
pub use instruction_decoder::encode;
pub use instructions::{Definition, Instruction, InstructionSet, Opcode};
pub use memory::MEMORY_SIZE;
pub use registers::{ACC_MAX, ACC_MIN};

use instruction_decoder::decode;
use memory::Memory;
use registers::Registers;

// Output is ASCII only, so cutting at a byte offset is safe.
const OUTPUT_LIMIT: usize = 16 * 1024;

pub struct Machine {
    pub cpu: Registers,
    pub previous: Registers, // registers before the last step
    pub memory: Memory,

    pub instruction_set: InstructionSet,

    pub entry_point: u16,

    pub halted: bool,
    pub waiting: bool,

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

    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        self.previous = self.cpu;

        self.cpu.mar = self.cpu.pc;

        let raw = self.memory.read(self.cpu.mar as usize);

        self.cpu.ir = raw as u16;

        self.cpu.pc = self.cpu.pc.wrapping_add(1);

        let instruction = decode(&self.instruction_set, raw);

        execute(self, instruction);
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
        let raw = self.memory.read(self.cpu.pc as usize);

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
        assert_eq!(machine.cpu.pc, 2);
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
        assert_eq!(machine.cpu.pc, 3);
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

    const MEMORY_SIZE_FOR_TESTS: usize = crate::machine::MEMORY_SIZE + 1;
}
