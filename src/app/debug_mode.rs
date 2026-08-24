use crate::debugger::debug::Debug as DebugState;
use egui::Ui;

use super::gui_state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState, debug: &mut DebugState) {
    ui.horizontal(|ui| {
        ui.monospace(">");
        let raw = ui.add(egui::TextEdit::singleline(&mut debug.command_input));

        if raw.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            debug.send(state, debug.command_input.clone());
            debug.command_input.clear();
        }
    });

    ui.separator();

    if ui.button("Help").clicked() {
        debug.help();
    }

    ui.separator();

    ui.vertical(|ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (line, valid) in &debug.command_history {
                    let color = if *valid {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    };

                    ui.label(egui::RichText::new(line).color(color));
                }
            });
    });
}
