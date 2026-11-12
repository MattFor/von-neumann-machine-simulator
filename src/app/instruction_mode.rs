use egui::Ui;

use super::gui_state::AppState;
use crate::machine::{Definition, InstructionSet};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Instruction Set");

    ui.horizontal(|ui| {
        if ui
            .button("Open Set")
            .on_hover_text("Load an instruction set file")
            .clicked()
        {
            state.open_instruction_set();
        }

        if ui
            .button("Save Set")
            .on_hover_text("Save this instruction set, micro steps included, to a file")
            .clicked()
        {
            state.save_instruction_set();
        }

        ui.separator();

        if ui
            .button("Add Instruction")
            .on_hover_text("Add an instruction under the next free code")
            .clicked()
        {
            let set = &mut state.machine.instruction_set;

            set.definitions.push(Definition {
                code: set.free_code(),
                ..Default::default()
            });
        }

        let presets: [(_, _, fn() -> InstructionSet); 3] = [
            (
                "Restore Default",
                "Back to the built-in instruction set",
                InstructionSet::default,
            ),
            (
                "Johnny",
                "The names and codes of the Johnny simulator: TAKE, ADD, SUB, SAVE, JMP, TST, INC, DEC, NULL, HLT",
                InstructionSet::johnny,
            ),
            (
                "Bonsai",
                "Johnny's minimal set: INC, DEC, JMP, TST, HLT",
                InstructionSet::bonsai,
            ),
        ];

        for (label, hint, preset) in presets {
            if ui.button(label).on_hover_text(hint).clicked() {
                state.machine.instruction_set = preset();
                state.status_message = format!(
                    "Loaded the {} instruction set",
                    state.machine.instruction_set.name
                );
            }
        }
    });

    ui.separator();

    crate::gui::instruction_set_view::show(ui, &mut state.machine);
}
