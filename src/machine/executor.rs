use super::{Machine, instructions::MicroOp};

pub fn execute(machine: &mut Machine, op: MicroOp) {
    let operand = machine.cpu.ir & 0xff;
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
        MicroOp::PcInc => machine.cpu.pc = machine.cpu.pc.wrapping_add(1),
        MicroOp::PcIncIfZero => {
            if acc == 0 {
                machine.cpu.pc = machine.cpu.pc.wrapping_add(1);
            }
        }

        MicroOp::AccZero => machine.cpu.set_acc(0),
        MicroOp::AccInc => machine.cpu.set_acc(acc.saturating_add(1)),
        MicroOp::AccDec => machine.cpu.set_acc(acc.saturating_sub(1)),
        MicroOp::Plus => machine.cpu.set_acc(acc.saturating_add(mbr)),
        MicroOp::Minus => machine.cpu.set_acc(acc.saturating_sub(mbr)),
        MicroOp::Times => machine.cpu.set_acc(acc.saturating_mul(mbr)),
        MicroOp::Divide => {
            if mbr == 0 {
                machine.print("error: division by zero\n");
                machine.halted = true;
            } else {
                machine.cpu.set_acc(acc / mbr);
            }
        }

        MicroOp::AccToDb => machine.cpu.set_mbr(acc),
        MicroOp::DbToAcc => machine.cpu.set_acc(mbr),

        MicroOp::Input => match machine.input.pop_front() {
            Some(value) => machine.cpu.set_acc(value),

            // The step is not counted, so it is tried again once a value arrives.
            None => machine.waiting = true,
        },

        MicroOp::Output => machine.print(&format!("{acc}\n")),
        MicroOp::Stop => machine.halted = true,
    }
}
