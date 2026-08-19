mod memory;
mod registers;
pub mod machine;
mod executor;
mod instructions;
mod instruction_decoder;

pub use instruction_decoder::decode;
pub use executor::execute;
pub use instructions::{Instruction, Opcode};