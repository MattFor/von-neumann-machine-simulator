use egui::{Key, KeyboardShortcut, Modifiers, Ui};
use std::time::Instant;

use super::gui_state::{AppState, MAX_SPEED, MIN_SPEED};
use crate::gui::action;

const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
const RUN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F5);
const STEP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F10);
const RESET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);

pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if action(ui, true, "Open Program", "Open a program file", OPEN) {
            state.open_program();
        }

        if action(
            ui,
            true,
            "Save Program",
            "Save the memory and the instruction set to a file",
            SAVE,
        ) {
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

        if action(
            ui,
            !state.running && !halted,
            "Run",
            "Run the program at the chosen speed",
            RUN,
        ) {
            state.last_tick = Instant::now();
            state.running = true;
        }

        if action(ui, state.running, "Pause", "Pause the running program", RUN) {
            state.running = false;
        }

        if action(ui, !halted, "Step", "Carry out one whole instruction", STEP) {
            state.step();
        }

        if action(
            ui,
            true,
            "Reset",
            "Put the registers and the console back to the start; memory is kept",
            RESET,
        ) {
            state.reset();
        }

        // A menu, so that wiping the whole memory takes a second click.
        ui.menu_button("Clear", |ui| {
            if ui.button("Erase all memory").clicked() {
                state.clear();
            }
        })
        .response
        .on_hover_text("Erase the whole memory");

        ui.separator();

        ui.label("Speed");

        ui.add(
            egui::Slider::new(&mut state.speed, MIN_SPEED..=MAX_SPEED)
                .logarithmic(true)
                .suffix(" ips"),
        )
        .on_hover_text("Instructions per second while running");
    });
}
