use super::{Machine, PcOverflow, instructions::MicroOp};

pub fn execute(machine: &mut Machine, op: MicroOp) {
    let operand = machine.instruction_set.split(machine.cpu.ir as i32).1 as u16;
    let (acc, mbr) = (machine.cpu.acc, machine.cpu.mbr);

    match op {
        MicroOp::DbToRam => machine.memory.write(machine.cpu.mar as usize, mbr),
        MicroOp::RamToDb => {
            let value = machine.memory.read(machine.cpu.mar as usize);

            machine.cpu.set_mbr(value);
        }

        MicroOp::DbToIns => machine.cpu.ir = mbr as u16,
        MicroOp::InsToAb => machine.cpu.mar = operand,
        MicroOp::InsToDb => machine.cpu.set_mbr(operand as i32),
        MicroOp::InsToPc => machine.cpu.pc = operand,
        MicroOp::InsToPcIfZero => {
            if acc == 0 {
                machine.cpu.pc = operand;
            }
        }

        MicroOp::InsToMc => {
            machine.executing = true;
            machine.mc = 0;
        }

        MicroOp::McZero => {
            machine.executing = false;
            machine.mc = 0;
        }

        MicroOp::PcToAb => machine.cpu.mar = machine.cpu.pc,
        MicroOp::PcInc => next_pc(machine),
        MicroOp::PcIncIfZero => {
            if acc == 0 {
                next_pc(machine);
            }
        }

        MicroOp::AccZero => machine.set_acc(0),
        MicroOp::AccInc => machine.set_acc(acc.saturating_add(1)),
        MicroOp::AccDec => machine.set_acc(acc.saturating_sub(1)),
        MicroOp::Plus => machine.set_acc(acc.saturating_add(mbr)),
        MicroOp::Minus => machine.set_acc(acc.saturating_sub(mbr)),
        MicroOp::Times => machine.set_acc(acc.saturating_mul(mbr)),
        MicroOp::Divide => {
            if mbr == 0 {
                machine.print("error: division by zero\n");
                machine.halted = true;
            } else {
                machine.set_acc(acc / mbr);
            }
        }

        MicroOp::AccToDb => machine.cpu.set_mbr(acc),
        MicroOp::DbToAcc => machine.set_acc(mbr),

        MicroOp::Input => match machine.input.pop_front() {
            Some(value) => machine.set_acc(value),

            // The step is not counted, so it is tried again once a value arrives.
            None => machine.waiting = true,
        },

        MicroOp::Output => machine.print(&format!("{acc}\n")),
        MicroOp::Stop => machine.halted = true,
    }
}

// Only the cells of the format exist; what comes after the last one is up to the format too.
fn next_pc(machine: &mut Machine) {
    let format = machine.instruction_set.format;
    let last = (format.cells() - 1) as u16;
    let pc = machine.cpu.pc;

    if pc < last {
        machine.cpu.pc = pc + 1;
        return;
    }

    match format.pc_overflow {
        PcOverflow::Wrap => machine.cpu.pc = 0,
        PcOverflow::Stay => machine.cpu.pc = last,
        PcOverflow::Halt => machine.halted = true,
    }
}
