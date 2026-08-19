pub struct Registers {
    pub acc: u16,   // accumulator
    pub pc: u16,    // program counter
    pub ir: u16,    // instruction register
    pub mar: u16,   // memory address register
    pub mbr: u16,   // memory buffer register
}

impl Registers {
    pub fn new() -> Self {
        Self {
            acc: 0,
            pc: 0,
            ir: 0,
            mar: 0,
            mbr: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
