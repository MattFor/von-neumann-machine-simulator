const VALUE_MIN: i32 = 0;
const VALUE_MAX: i32 = 19_999;

// MBR and ACC should the have same boundaries!
const MBR_MIN: i32 = VALUE_MIN; // unused: should be used when implementing setter for mbr
const MBR_MAX: i32 = VALUE_MAX; // unused: should be used when implementing setter for mbr

pub const ACC_MIN: i32 = VALUE_MIN;
pub const ACC_MAX: i32 = VALUE_MAX;


pub struct Registers {
    pub acc: i32,   // accumulator
    pub pc: u16,    // program counter
    pub ir: u16,    // instruction register
    pub mar: u16,   // memory address register
    pub mbr: i32,   // memory buffer register
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
