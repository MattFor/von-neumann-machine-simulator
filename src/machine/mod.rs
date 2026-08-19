mod executor;
mod instruction_decoder;
mod instructions;
pub mod machine;
mod memory;
mod registers;

pub use executor::execute;
pub use instruction_decoder::decode;
pub use instructions::{Instruction, Opcode};
