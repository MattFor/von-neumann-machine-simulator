const VALUE_MIN: i32 = 0;
const VALUE_MAX: i32 = 19_999;

// The accumulator range of the presets; a format can set its own.
pub const ACC_MIN: i32 = VALUE_MIN;
pub const ACC_MAX: i32 = VALUE_MAX;

#[derive(Clone, Copy, PartialEq)]
pub struct Registers {
    pub acc: i32, // accumulator
    pub pc: u16,  // program counter
    pub ir: u16,  // instruction register
    pub mar: u16, // memory address register
    pub mbr: i32, // memory buffer register
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
