use super::{executor::execute, instruction_decoder::decode, memory::Memory, registers::Registers};

pub struct Machine {
    pub cpu: Registers,
    pub memory: Memory,

    pub entry_point: u16,

    pub halted: bool,

    pub output: String,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: Registers::new(),
            memory: Memory::new(),

            entry_point: 0,

            halted: false,

            output: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.pc = self.entry_point;

        self.halted = false;
        self.output.clear();
    }

    pub fn clear(&mut self) {
        self.memory.reset();
        self.entry_point = 0;

        self.reset();
    }

    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        self.cpu.mar = self.cpu.pc;

        let raw = self.memory.read(self.cpu.mar as usize);

        self.cpu.ir = raw as u16;

        self.cpu.pc = self.cpu.pc.wrapping_add(1);

        let instruction = decode(raw);

        execute(self, instruction);
    }

    pub fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            if self.halted {
                break;
            }

            self.step();
        }
    }

    pub fn current_instruction(&self) -> String {
        let raw = self.memory.read(self.cpu.pc as usize);
        let instruction = decode(raw);

        format!(
            "{:04X}  {:?} {}",
            raw as u16, instruction.opcode, instruction.operand
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::registers::ACC_MAX;

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
        let mut machine = machine_with(&[word(3, 200), word(5, 255), word(5, 255)]);

        machine.run_steps(3);

        assert_eq!(machine.cpu.acc, ACC_MAX);
    }

    #[test]
    fn subtract_does_not_go_below_zero() {
        let mut machine = machine_with(&[word(4, 10)]);

        machine.step();

        assert_eq!(machine.cpu.acc, 0);
    }

    #[test]
    fn store_and_load_move_through_the_buffer_register() {
        let mut machine = machine_with(&[word(3, 42), word(2, 100), word(1, 100), word(255, 0)]);

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
        let mut machine = machine_with(&[word(3, 1), word(255, 0), word(3, 1)]);

        machine.run_steps(100);

        assert!(machine.halted);
        assert_eq!(machine.cpu.acc, 1);
        assert_eq!(machine.cpu.pc, 2);
    }

    #[test]
    fn reset_keeps_memory_and_returns_to_entry_point() {
        let mut machine = machine_with(&[word(3, 5), word(255, 0)]);
        machine.entry_point = 0;

        machine.run_steps(8);
        machine.reset();

        assert!(!machine.halted);
        assert_eq!(machine.cpu.pc, 0);
        assert_eq!(machine.cpu.acc, 0);
        assert_eq!(machine.memory.read(0), word(3, 5));
    }

    #[test]
    fn countdown_asset_runs_to_completion() {
        let raw = std::fs::read_to_string("assets/countdown.json").unwrap();
        let program: crate::program::program::Program = serde_json::from_str(&raw).unwrap();

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

    const MEMORY_SIZE_FOR_TESTS: usize = crate::machine::MEMORY_SIZE + 1;
}
