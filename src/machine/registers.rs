// The accumulator range of the presets; a format can set its own.
pub const ACC_MIN: i32 = 0;
pub const ACC_MAX: i32 = 19_999;

#[derive(Clone, Copy, PartialEq)]
pub struct Registers {
    pub acc: i32, // accumulator
    pub pc: u16,  // program counter
    pub ir: u16,  // instruction register
    pub mar: u16, // memory address register
    pub mbr: i32, // memory buffer register

    // Set by plus and minus on a format with flags only.
    pub carry: bool,
    pub zero: bool,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            acc: 0,
            pc: 0,
            ir: 0,
            mar: 0,
            mbr: 0,
            carry: false,
            zero: false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
