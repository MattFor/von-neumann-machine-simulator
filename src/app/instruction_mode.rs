use egui::Ui;

use super::gui_state::AppState;
use crate::machine::{Definition, InstructionSet};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Instruction Set");

    ui.horizontal(|ui| {
        if ui
            .button("Open Set")
            .on_hover_text("Load an instruction set file, or a Johnny .mc microcode file")
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

        let free = state.machine.instruction_set.free_code();

        if ui
            .add_enabled(free.is_some(), egui::Button::new("Add Instruction"))
            .on_hover_text("Add an instruction under the next free code")
            .on_disabled_hover_text("Every code is taken")
            .clicked()
            && let Some(code) = free
        {
            state.machine.instruction_set.definitions.push(Definition {
                code,
                ..Default::default()
            });
        }

        // In the order of InstructionSet::presets.
        let buttons = [
            (
                "Restore Default",
                "Standard mode with the built-in instruction set",
            ),
            (
                "Johnny",
                "Johnny mode with the instructions of the Johnny simulator: TAKE, ADD, SUB, SAVE, JMP, TST, INC, DEC, NULL, HLT",
            ),
            (
                "Bonsai",
                "Johnny mode with its minimal set: INC, DEC, JMP, TST, HLT",
            ),
        ];

        for ((label, hint), preset) in buttons.into_iter().zip(InstructionSet::presets()) {
            if ui.button(label).on_hover_text(hint).clicked() {
                state.use_set(preset);
            }
        }
    });

    ui.separator();

    crate::gui::instruction_set_view::show(ui, &mut state.machine);
}
