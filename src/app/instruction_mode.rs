use egui::Ui;

use super::gui_state::AppState;
use crate::machine::{Definition, InstructionSet};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Instruction Set");

    ui.horizontal(|ui| {
        if ui.button("Open Set").clicked() {
            state.open_instruction_set();
        }

        if ui.button("Save Set").clicked() {
            state.save_instruction_set();
        }

        ui.separator();

        if ui.button("Add Instruction").clicked() {
            state
                .machine
                .instruction_set
                .definitions
                .push(Definition::default());
        }

        if ui.button("Restore Default").clicked() {
            state.machine.instruction_set = InstructionSet::default();
            state.status_message = "Restored default instruction set".to_string();
        }
    });

    ui.separator();

    crate::gui::instruction_set_view::show(ui, &mut state.machine);
}
