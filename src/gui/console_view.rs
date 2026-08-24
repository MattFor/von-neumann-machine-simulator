use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.heading("Console");

        if ui.button("Clear").clicked() {
            machine.output.clear();
        }
    });

    egui::ScrollArea::vertical()
        .id_salt("console_scroll")
        .max_height(160.0)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if machine.output.is_empty() {
                ui.weak("(no output)");
            } else {
                ui.monospace(&machine.output);
            }
        });
}
