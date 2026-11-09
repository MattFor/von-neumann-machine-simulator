use egui::{Key, KeyboardShortcut, Modifiers, Ui};
use std::time::Instant;

use super::gui_state::{AppState, MAX_SPEED, MIN_SPEED};

const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
const RUN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F5);
const STEP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F10);
const RESET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);

// A button that also fires on its keyboard shortcut.
fn action(ui: &mut Ui, enabled: bool, label: &str, shortcut: KeyboardShortcut) -> bool {
    let pressed = enabled && ui.input_mut(|input| input.consume_shortcut(&shortcut));

    ui.add_enabled(enabled, egui::Button::new(label))
        .on_hover_text(ui.ctx().format_shortcut(&shortcut))
        .clicked()
        || pressed
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if action(ui, true, "Open Program", OPEN) {
            state.open_program();
        }

        if action(ui, true, "Save Program", SAVE) {
            state.save_program();
        }

        if ui
            .button("Example")
            .on_hover_text("Load a bundled countdown program")
            .clicked()
        {
            state.load_example();
        }

        ui.separator();

        let halted = state.halted();

        if action(ui, !state.running && !halted, "Run", RUN) {
            state.last_tick = Instant::now();
            state.running = true;
        }

        if action(ui, state.running, "Pause", RUN) {
            state.running = false;
        }

        if action(ui, !halted, "Step", STEP) {
            state.step();
        }

        if action(ui, true, "Reset", RESET) {
            state.reset();
        }

        // A menu, so that wiping the whole memory takes a second click.
        ui.menu_button("Clear", |ui| {
            if ui.button("Erase all memory").clicked() {
                state.clear();
            }
        });

        ui.separator();

        ui.label("Speed");

        ui.add(
            egui::Slider::new(&mut state.speed, MIN_SPEED..=MAX_SPEED)
                .logarithmic(true)
                .suffix(" ips"),
        );
    });
}
