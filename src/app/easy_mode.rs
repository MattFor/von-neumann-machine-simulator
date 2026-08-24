use egui::Ui;

use super::{controls, gui_state::AppState};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Von Neumann Machine Simulator");

    controls::show(ui, state);

    ui.separator();

    ui.label(format!("Program Counter: {}", state.machine.cpu.pc));

    ui.label(format!(
        "Instruction: {}",
        state.machine.current_instruction()
    ));

    ui.label(format!("Accumulator: {}", state.machine.cpu.acc));

    ui.separator();

    crate::gui::console_view::show(ui, &mut state.machine);
}
