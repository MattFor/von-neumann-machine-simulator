use egui::Ui;

use super::gui_state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("Run").clicked() {
            state.running = true;
        }

        if ui.button("Pause").clicked() {
            state.running = false;
        }

        if ui.button("Step").clicked() {
            state.machine.step();
        }

        if ui.button("Reset").clicked() {
            state.machine.reset();
        }
    });

    ui.separator();

    ui.columns(3, |columns| {
        crate::gui::memory_view::show(&mut columns[0], &mut state.machine);

        crate::gui::register_view::show(&mut columns[1], &state.machine);

        crate::gui::cpu_view::show(&mut columns[2], &mut state.machine);
    });

    ui.separator();

    crate::gui::console_view::show(ui, &state.machine);
}
