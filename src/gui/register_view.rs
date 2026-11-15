use egui::Ui;

use crate::machine::Machine;

pub fn show(ui: &mut Ui, machine: &Machine) {
    ui.heading("Registers");

    egui::Grid::new("register_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Reg");
            ui.label("Dec");
            // Decimal is always there; a machine in another radix gets a column for it too.
            let format = machine.instruction_set.format;
            let radix = format.radix != 10;

            if radix {
                ui.label(if format.radix == 16 {
                    "Hex".to_string()
                } else {
                    format!("Base {}", format.radix)
                });
            }

            ui.end_row();

            for (name, description, value, previous, shown, digits) in rows(machine) {
                // Registers the last step changed stand out.
                let color = if value != previous {
                    ui.visuals().warn_fg_color
                } else {
                    ui.visuals().text_color()
                };
                let text = |text: String| egui::RichText::new(text).monospace().color(color);

                ui.monospace(name).on_hover_text(description);
                ui.label(text(format!("{value}")));
                if radix {
                    ui.label(text(format.text(shown, digits)));
                }

                ui.end_row();
            }
        });

    ui.separator();

    ui.label(format!(
        "State: {}",
        if machine.halted {
            "halted"
        } else if machine.waiting {
            "waiting for input"
        } else {
            "ready"
        }
    ));

    ui.label(format!("Next: {}", machine.current_instruction()))
        .on_hover_text("The instruction the program counter points at");
}

// Name, description, value, value before the last step, the word shown in the radix, its digits.
type Row = (&'static str, &'static str, i32, i32, i32, u32);

// The registers, then the flags on a machine that has them.
fn rows(machine: &Machine) -> Vec<Row> {
    let format = machine.instruction_set.format;
    let (cpu, old) = (&machine.cpu, &machine.previous);
    let word = format.code_digits + format.operand_digits;
    let flags = [
        (
            "CF",
            "Carry flag: the last plus went past the top, or the last minus borrowed nothing; JC \
             and cf:ins->pc jump on it",
            cpu.carry as i32,
            old.carry as i32,
            cpu.carry as i32,
            1,
        ),
        (
            "ZF",
            "Zero flag: the last plus or minus gave zero; JZ and zf:ins->pc jump on it",
            cpu.zero as i32,
            old.zero as i32,
            cpu.zero as i32,
            1,
        ),
    ];

    // Numbers are shown in the radix as the word that holds them, so -1 is FFFF on MARIE.
    [
        (
            "ACC",
            "Accumulator: where all arithmetic happens",
            cpu.acc,
            old.acc,
            format.word(cpu.acc),
            word,
        ),
        (
            "PC",
            "Program counter: address of the next instruction",
            cpu.pc as i32,
            old.pc as i32,
            cpu.pc as i32,
            format.operand_digits,
        ),
        (
            "IR",
            "Instruction register: the instruction being carried out",
            cpu.ir as i32,
            old.ir as i32,
            cpu.ir as i32,
            word,
        ),
        (
            "MAR",
            "Memory address register: the address bus, it picks a memory cell",
            cpu.mar as i32,
            old.mar as i32,
            cpu.mar as i32,
            format.operand_digits,
        ),
        (
            "MBR",
            "Memory buffer register: the data bus, it carries a value to or from memory",
            cpu.mbr,
            old.mbr,
            format.word(cpu.mbr),
            word,
        ),
    ]
    .into_iter()
    .chain(flags.into_iter().filter(|_| format.flags))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{InstructionSet, MicroOp};

    #[test]
    fn flags_show_on_a_machine_with_flags_and_stand_out_when_they_change() {
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::sap1();
        machine.set_acc(255);
        machine.cpu.mbr = 1;
        machine.manual(MicroOp::Plus);

        let flags: Vec<_> = rows(&machine)
            .into_iter()
            .filter(|row| row.0.ends_with('F'))
            .map(|(name, _, value, previous, ..)| (name, value, previous))
            .collect();

        assert_eq!(flags, [("CF", 1, 0), ("ZF", 1, 0)]);

        machine.instruction_set = InstructionSet::lmc();

        assert_eq!(rows(&machine).len(), 5);
    }
}
