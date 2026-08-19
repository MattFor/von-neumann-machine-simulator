use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &Machine) {
    ui.label(format!("Accumulator: {}", machine.cpu.acc));
}