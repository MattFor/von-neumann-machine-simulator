use egui::Ui;

use super::gui_state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Von Neumann Machine Simulator");

    ui.horizontal(|ui| {
        if ui.button("Open Program").clicked() {}

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

    ui.label(format!("Program Counter: {}", state.machine.cpu.pc));

    ui.label(format!(
        "Instruction: {}",
        state.machine.current_instruction()
    ));

    ui.separator();

    ui.heading("Output");

    ui.add(
        egui::TextEdit::multiline(&mut state.machine.output)
            .desired_rows(10)
            .interactive(false),
    );
}
