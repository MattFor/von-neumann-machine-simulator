use egui::Ui;

use crate::machine::Machine;

pub fn show(ui: &mut Ui, machine: &Machine) {
    ui.heading("Registers");

    egui::Grid::new("register_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Reg");
            ui.label("Dec");
            // Hex means nothing to a Johnny program.
            let hex = !machine.instruction_set.johnny;

            if hex {
                ui.label("Hex");
            }

            ui.end_row();

            let (cpu, old) = (&machine.cpu, &machine.previous);

            for (name, description, value, previous) in [
                (
                    "ACC",
                    "Accumulator: where all arithmetic happens",
                    cpu.acc,
                    old.acc,
                ),
                (
                    "PC",
                    "Program counter: address of the next instruction",
                    cpu.pc as i32,
                    old.pc as i32,
                ),
                (
                    "IR",
                    "Instruction register: the instruction being carried out",
                    cpu.ir as i32,
                    old.ir as i32,
                ),
                (
                    "MAR",
                    "Memory address register: the address bus, it picks a memory cell",
                    cpu.mar as i32,
                    old.mar as i32,
                ),
                (
                    "MBR",
                    "Memory buffer register: the data bus, it carries a value to or from memory",
                    cpu.mbr,
                    old.mbr,
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
                if hex {
                    ui.label(text(format!("{:04X}", value as u16)));
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
