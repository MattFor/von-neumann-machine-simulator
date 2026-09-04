use super::{controls, gui_state::AppState};
use egui::{Color32, Ui};

#[cfg(feature = "debug-mode")]
use egui::{Frame, Stroke};

const CONSOLE_HEIGHT: f32 = 100.0;
const FORCE_HIDE_CONSOLE_AT: f32 = 300.0; // Force hide console when height is lower than this value. Prevents visual bug.

#[cfg(feature = "debug-mode")]
fn debug_view(ui: &mut Ui, color: Color32, add: impl FnOnce(&mut Ui)) {
    Frame::new()
        .stroke(Stroke::new(1.0, color))
        .inner_margin(4.0)
        .show(ui, |ui| {
            add(ui);
        });
}

#[cfg(not(feature = "debug-mode"))]
fn debug_view(ui: &mut Ui, _color: Color32, add: impl FnOnce(&mut Ui)) {
    add(ui);
}

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
                        debug_view(&mut columns[column], Color32::RED, |ui| {
                            crate::gui::memory_view::show(ui, &mut state.machine);
                        });
                        column += 1;
                    }

                    if state.show_registers {
                        debug_view(&mut columns[column], Color32::RED, |ui| {
                            crate::gui::register_view::show(ui, &state.machine);
                        });
                        column += 1;
                    }

                    if state.show_cpu {
                        debug_view(&mut columns[column], Color32::RED, |ui| {
                            crate::gui::cpu_view::show(ui, &mut state.machine);
                        });
                    }
                });
            },
        );
    }

    if state.show_console {
        ui.separator();

        debug_view(ui, Color32::RED, |ui| {
            crate::gui::console_view::show(ui, &mut state.machine); // NOTE: change it to resizable bottom panel if possible
        });
    }
}
