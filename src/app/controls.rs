use egui::Ui;

use super::gui_state::{AppState, MAX_SPEED, MIN_SPEED};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("Open Program").clicked() {
            state.open_program();
        }

        if ui.button("Save Program").clicked() {
            state.save_program();
        }

        ui.separator();

        let halted = state.halted();

        if ui
            .add_enabled(!state.running && !halted, egui::Button::new("Run"))
            .clicked()
        {
            state.running = true;
            state.status_message = "Running".to_string();
        }

        if ui
            .add_enabled(state.running, egui::Button::new("Pause"))
            .clicked()
        {
            state.running = false;
            state.status_message = "Paused".to_string();
        }

        if ui.add_enabled(!halted, egui::Button::new("Step")).clicked() {
            state.step();
        }

        if ui.button("Reset").clicked() {
            state.reset();
        }

        if ui.button("Clear").clicked() {
            state.clear();
        }

        ui.separator();

        ui.label("Speed");

        ui.add(
            egui::Slider::new(&mut state.speed, MIN_SPEED..=MAX_SPEED)
                .logarithmic(true)
                .suffix(" ips"),
        );
    });
}
