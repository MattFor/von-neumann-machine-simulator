use egui::Ui;

use super::{controls, gui_state::AppState};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    controls::show(ui, state);

    ui.horizontal(|ui| {
        ui.label("Views:");
        ui.checkbox(&mut state.show_memory, "Memory");
        ui.checkbox(&mut state.show_registers, "Registers");
        ui.checkbox(&mut state.show_cpu, "CPU");
        ui.checkbox(&mut state.show_console, "Console");
    });

    ui.separator();

    let panels = [state.show_memory, state.show_registers, state.show_cpu]
        .iter()
        .filter(|shown| **shown)
        .count();

    if panels > 0 {
        ui.columns(panels, |columns| {
            let mut column = 0;

            if state.show_memory {
                crate::gui::memory_view::show(&mut columns[column], &mut state.machine);
                column += 1;
            }

            if state.show_registers {
                crate::gui::register_view::show(&mut columns[column], &state.machine);
                column += 1;
            }

            if state.show_cpu {
                crate::gui::cpu_view::show(&mut columns[column], &mut state.machine);
            }
        });
    }

    if state.show_console {
        ui.separator();

        crate::gui::console_view::show(ui, &mut state.machine);
    }
}
