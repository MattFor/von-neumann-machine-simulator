use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &Machine) {
    ui.horizontal(|ui| {
        if ui.button("acc++").clicked() {

        }

        if ui.button("acc--").clicked() {
            
        }
    });
}