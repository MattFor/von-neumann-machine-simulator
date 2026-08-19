use super::{
    instructions::{Instruction, Opcode},
    machine::Machine,
    registers::{Registers, ACC_MAX, ACC_MIN},
};

pub fn execute(machine: &mut Machine, instruction: Instruction) {
    match instruction.opcode {
        Opcode::Nop => {}

        Opcode::Load => {
            machine.cpu.acc = machine.memory.read(instruction.operand as usize);
        }

        Opcode::Store => {
            machine
                .memory
                .write(instruction.operand as usize, machine.cpu.acc);
        }

        Opcode::Add => {
            machine.cpu.acc += instruction.operand;

            if machine.cpu.acc > ACC_MAX {
                machine.cpu.acc = ACC_MAX;
            }
        }

        Opcode::Sub => {
            machine.cpu.acc -= instruction.operand;

            if machine.cpu.acc < ACC_MIN {
                machine.cpu.acc = ACC_MIN;
            }
        }

        Opcode::Mul => {
            machine.cpu.acc *= instruction.operand;
        }

        Opcode::Div => {
            if instruction.operand != 0 {
                machine.cpu.acc /= instruction.operand;
            }
        }

        Opcode::Jump => {
            machine.cpu.pc = instruction.operand as u16;
        }

        Opcode::JumpIfZero => {
            if machine.cpu.acc == 0 {
                machine.cpu.pc = instruction.operand as u16;
            }
        }

        Opcode::Output => {
            machine.output.push_str(&format!("{}\n", machine.cpu.acc));
        }

        Opcode::Halt => {
            machine.halted = true;
        }

        Opcode::Input => {
            // Handle input later
        }
    }
}
