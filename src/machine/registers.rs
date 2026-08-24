const VALUE_MIN: i32 = 0;
const VALUE_MAX: i32 = 19_999;

const MBR_MIN: i32 = VALUE_MIN;
const MBR_MAX: i32 = VALUE_MAX;

pub const ACC_MIN: i32 = VALUE_MIN;
pub const ACC_MAX: i32 = VALUE_MAX;

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

    pub fn set_acc(&mut self, value: i32) {
        self.acc = value.clamp(ACC_MIN, ACC_MAX);
    }

    pub fn set_mbr(&mut self, value: i32) {
        self.mbr = value.clamp(MBR_MIN, MBR_MAX);
    }
}
