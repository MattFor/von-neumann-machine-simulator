use super::{executor::execute, instruction_decoder::decode, memory::Memory, registers::Registers};

pub struct Machine {
    pub cpu: Registers,
    pub memory: Memory,

    pub halted: bool,

    pub output: String,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: Registers::new(),
            memory: Memory::new(),

            halted: false,

            output: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory.reset();

        self.halted = false;
        self.output.clear();
    }

    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        let raw = self.memory.read(self.cpu.pc as usize);

        self.cpu.ir = raw as u16;

        self.cpu.pc += 1;

        let instruction = decode(raw);

        execute(self, instruction);
    }

    pub fn current_instruction(&self) -> String {
        let raw = self.memory.read(self.cpu.pc as usize);

        format!("{raw:04X}")
    }
}
