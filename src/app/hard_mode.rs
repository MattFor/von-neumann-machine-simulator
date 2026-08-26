use egui::Ui;

use super::{controls, gui_state::AppState};

const CONSOLE_HEIGHT: f32 = 100.0;
const FORCE_HIDE_CONSOLE_AT: f32 = 300.0; // Force hide console when height is lower than this value. Prevents visual bug.

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

    let width = ui.available_width();
    // NOTE: change the check in future when blessed with greater idea
    let panels_height = if state.show_console && ui.available_height() > FORCE_HIDE_CONSOLE_AT {
        (ui.available_height() - CONSOLE_HEIGHT).max(0.0)
    } else {
        ui.available_height()
    };

    if panels > 0 {
        ui.allocate_ui_with_layout(
            egui::vec2(width, panels_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
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
            },
        );
    }

    if state.show_console {
        ui.separator();

        crate::gui::console_view::show(ui, &mut state.machine); // NOTE: change it to resizable bottom panel if possible
    }
}
