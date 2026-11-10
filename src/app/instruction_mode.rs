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
            let set = &mut state.machine.instruction_set;
            let code = (0..=255).find(|code| set.find(*code).is_none());

            set.definitions.push(Definition {
                code: code.unwrap_or(0),
                ..Default::default()
            });
        }

        if ui.button("Restore Default").clicked() {
            state.machine.instruction_set = InstructionSet::default();
            state.status_message = "Restored default instruction set".to_string();
        }
    });

    ui.separator();

    crate::gui::instruction_set_view::show(ui, &mut state.machine);
}
