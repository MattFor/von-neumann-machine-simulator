pub const MEMORY_SIZE: usize = 65_536;

pub struct Memory {
    data: Vec<i32>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: vec![0; MEMORY_SIZE],
        }
    }

    pub fn read(&self, address: usize) -> i32 {
        self.data[address]
    }

    pub fn write(&mut self, address: usize, value: i32) {
        self.data[address] = value;
    }

    pub fn reset(&mut self) {
        self.data.fill(0);
    }

    pub fn data(&self) -> &[i32] {
        &self.data
    }
}
