use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &Machine) {
    ui.heading("Registers");

    egui::Grid::new("register_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Reg");
            ui.label("Dec");
            ui.label("Hex");
            ui.end_row();

            let cpu = &machine.cpu;

            for (name, value) in [
                ("ACC", cpu.acc),
                ("PC", cpu.pc as i32),
                ("IR", cpu.ir as i32),
                ("MAR", cpu.mar as i32),
                ("MBR", cpu.mbr),
            ] {
                ui.monospace(name);
                ui.monospace(format!("{value}"));
                ui.monospace(format!("{:04X}", value as u16));
                ui.end_row();
            }
        });

    ui.separator();

    ui.label(format!(
        "State: {}",
        if machine.halted { "halted" } else { "ready" }
    ));

    ui.label(format!("Next: {}", machine.current_instruction()));
}
