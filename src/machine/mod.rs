mod executor;
mod instruction_decoder;
mod instructions;
pub mod machine;
mod memory;
mod registers;

pub use executor::execute;
pub use instruction_decoder::{decode, encode};
pub use instructions::{Instruction, Opcode};
pub use memory::MEMORY_SIZE;
pub use registers::{ACC_MAX, ACC_MIN};
