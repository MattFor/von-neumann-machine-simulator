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

            let (cpu, old) = (&machine.cpu, &machine.previous);
            let word = format.code_digits + format.operand_digits;

            // Numbers are shown in the radix as the word that holds them, so -1 is FFFF on MARIE.
            for (name, description, value, previous, shown, digits) in [
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
            ] {
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
