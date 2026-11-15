use super::{Machine, PcOverflow, instructions::MicroOp};

pub fn execute(machine: &mut Machine, op: MicroOp) {
    let operand = machine.instruction_set.split(machine.cpu.ir as i32).1 as u16;
    let (acc, mbr) = (machine.cpu.acc, machine.cpu.mbr);

    match op {
        MicroOp::DbToRam => machine.write_data_bus(),
        MicroOp::RamToDb => {
            let address = machine.address(machine.cpu.mar as i32);
            machine.cpu.mar = address;
            let value = machine.memory.read(address as usize);

            machine.memory_to_data_bus(value);
        }

        MicroOp::DbToIns => machine.cpu.ir = mbr as u16,
        MicroOp::InsToAb => machine.cpu.mar = machine.address(operand as i32),
        MicroOp::InsToDb => machine.set_data_bus(operand as i32),
        MicroOp::InsToPc => machine.cpu.pc = machine.address(operand as i32),
        MicroOp::InsToPcIfZero => {
            if acc == 0 {
                machine.cpu.pc = machine.address(operand as i32);
            }
        }
        MicroOp::InsToPcIfNonNegative => {
            if acc >= 0 {
                machine.cpu.pc = machine.address(operand as i32);
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

        MicroOp::PcToAb => {
            machine.normalize_pc();
            machine.cpu.mar = machine.cpu.pc;
        }
        MicroOp::PcToDb => {
            machine.normalize_pc();
            machine.set_data_bus(machine.cpu.pc as i32);
        }
        MicroOp::PcInc => next_pc(machine),
        MicroOp::PcIncIfZero => {
            if acc == 0 {
                next_pc(machine);
            }
        }
        MicroOp::PcIncIfPositive => {
            if acc > 0 {
                next_pc(machine);
            }
        }
        MicroOp::PcIncIfNegative => {
            if acc < 0 {
                next_pc(machine);
            }
        }
        MicroOp::DbToAb => machine.cpu.mar = machine.address(mbr),
        MicroOp::AbToPc => machine.cpu.pc = machine.address(machine.cpu.mar as i32),

        MicroOp::AccZero => machine.set_acc(0),
        MicroOp::AccInc => machine.set_arithmetic_acc(i64::from(acc) + 1),
        MicroOp::AccDec => machine.set_arithmetic_acc(i64::from(acc) - 1),
        MicroOp::Plus => machine.set_arithmetic_acc(i64::from(acc) + i64::from(mbr)),
        MicroOp::Minus => machine.set_arithmetic_acc(i64::from(acc) - i64::from(mbr)),
        MicroOp::Times => machine.set_arithmetic_acc(i64::from(acc) * i64::from(mbr)),
        MicroOp::Divide => {
            if mbr == 0 {
                machine.print("error: division by zero\n");
                machine.halted = true;
            } else {
                machine.set_arithmetic_acc(i64::from(acc) / i64::from(mbr));
            }
        }

        MicroOp::AccToDb => machine.set_data_bus(acc),
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
    let last = (format.cells().max(1) - 1) as u16;
    machine.normalize_pc();
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
