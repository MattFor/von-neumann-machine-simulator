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
        self.data.get(address).copied().unwrap_or(0)
    }

    pub fn write(&mut self, address: usize, value: i32) {
        if let Some(cell) = self.data.get_mut(address) {
            *cell = value;
        }
    }

    pub fn load(&mut self, values: &[i32]) {
        self.reset();

        let length = values.len().min(MEMORY_SIZE);

        self.data[..length].copy_from_slice(&values[..length]);
    }

    pub fn reset(&mut self) {
        self.data.fill(0);
    }

    pub fn data(&self) -> &[i32] {
        &self.data
    }
}
