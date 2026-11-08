use super::{
    instructions::{Instruction, Opcode},
    machine::Machine,
    registers::ACC_MAX,
};

pub fn execute(machine: &mut Machine, instruction: Instruction) {
    match instruction.opcode {
        Opcode::Nop => {}

        Opcode::Load => {
            machine.cpu.mar = instruction.operand as u16;

            let value = machine.memory.read(machine.cpu.mar as usize);

            machine.cpu.set_mbr(value);
            machine.cpu.set_acc(machine.cpu.mbr);
        }

        Opcode::Store => {
            machine.cpu.mar = instruction.operand as u16;

            machine.cpu.set_mbr(machine.cpu.acc);
            machine
                .memory
                .write(machine.cpu.mar as usize, machine.cpu.mbr);
        }

        Opcode::Add => {
            machine.cpu.mar = instruction.operand as u16;
            let value = machine.memory.read(machine.cpu.mar as usize);
            machine.cpu.set_mbr(value);
            machine
                .cpu
                .set_acc(machine.cpu.acc.saturating_add(machine.cpu.mbr));
        }

        Opcode::Sub => {
            machine.cpu.mar = instruction.operand as u16;
            let value = machine.memory.read(machine.cpu.mar as usize);
            machine.cpu.set_mbr(value);
            machine
                .cpu
                .set_acc(machine.cpu.acc.saturating_sub(machine.cpu.mbr));
        }

        Opcode::Mul => {
            machine
                .cpu
                .set_acc(machine.cpu.acc.saturating_mul(instruction.operand));
        }

        Opcode::Div => {
            if instruction.operand == 0 {
                machine.output.push_str("error: division by zero\n");
                machine.halted = true;
            } else {
                machine.cpu.set_acc(machine.cpu.acc / instruction.operand);
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

        Opcode::Inc => {
            machine.cpu.mar = instruction.operand as u16;
            let value = machine.memory.read(machine.cpu.mar as usize);
            machine.cpu.set_mbr(value);

            let updated = machine.cpu.mbr.saturating_add(1).min(ACC_MAX);
            machine.cpu.set_mbr(updated);
            machine
                .memory
                .write(machine.cpu.mar as usize, machine.cpu.mbr);
        }

        Opcode::Dec => {
            machine.cpu.mar = instruction.operand as u16;
            let value = machine.memory.read(machine.cpu.mar as usize);
            machine.cpu.set_mbr(value);

            let updated = machine.cpu.mbr.saturating_sub(1).max(0);
            machine.cpu.set_mbr(updated);
            machine
                .memory
                .write(machine.cpu.mar as usize, machine.cpu.mbr);
        }

        Opcode::Null => {
            machine.cpu.mar = instruction.operand as u16;
            machine.cpu.set_mbr(0);
            machine
                .memory
                .write(machine.cpu.mar as usize, machine.cpu.mbr);
        }

        Opcode::Tst => {
            machine.cpu.mar = instruction.operand as u16;
            let value = machine.memory.read(machine.cpu.mar as usize);
            machine.cpu.set_mbr(value);

            if machine.cpu.mbr == 0 {
                machine.cpu.pc = machine.cpu.pc.wrapping_add(1);
            }
        }

        Opcode::Halt => {
            machine.halted = true;
        }

        Opcode::Input => match machine.input.pop_front() {
            Some(value) => machine.cpu.set_acc(value),

            None => {
                machine.waiting = true;
                machine.cpu.pc = machine.cpu.pc.wrapping_sub(1);
            }
        },
    }
}
