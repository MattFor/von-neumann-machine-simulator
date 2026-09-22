use super::{controls, gui_state::AppState};
use crate::gui::data_bus_view;
use egui::{Color32, Ui};

#[cfg(feature = "debug-mode")]
use egui::{Frame, Stroke};

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

    let cpu_connections = ui.painter().add(egui::Shape::Noop);
    let transfers = egui::Panel::bottom("data_bus_panel")
        .exact_size(data_bus_view::HEIGHT)
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show(ui, |ui| {
            data_bus_view::show(ui, &mut state.machine, state.show_memory, state.show_cpu)
        })
        .inner;
    let mut accumulator = None;

    let panels = [state.show_memory, state.show_registers, state.show_cpu]
        .iter()
        .filter(|shown| **shown)
        .count();

    let width = ui.available_width();
    let panels_height = ui.available_height();

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
                            egui::ScrollArea::vertical()
                                .id_salt("registers_scroll")
                                .show(ui, |ui| {
                                    crate::gui::register_view::show(ui, &state.machine);
                                });
                        });
                        column += 1;
                    }

                    if state.show_cpu {
                        debug_view(&mut columns[column], Color32::RED, |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("cpu_scroll")
                                .show(ui, |ui| {
                                    accumulator =
                                        crate::gui::cpu_view::show(ui, &mut state.machine);
                                });
                        });
                    }
                });
            },
        );
    }

    if state.show_cpu {
        ui.painter().set(
            cpu_connections,
            data_bus_view::cpu_connections(transfers, accumulator),
        );
    }
}
