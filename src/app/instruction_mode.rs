use egui::Ui;

use super::gui_state::{AppState, PRESETS};
use crate::machine::Definition;

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

        // The same as the machine picker, so that this one's set can be put back here too.
        for (preset, (label, hint)) in PRESETS.into_iter().enumerate() {
            if ui
                .button(label)
                .on_hover_text(format!(
                    "{hint}. Switches to this machine, or gives it its own set back"
                ))
                .clicked()
            {
                state.pick(preset);
            }
        }
    });

    ui.separator();

    crate::gui::format_view::show(ui, &mut state.machine.instruction_set);
    state.machine.fit_entry_point();

    ui.separator();

    crate::gui::instruction_set_view::show(ui, &mut state.machine);
}
