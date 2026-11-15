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
    ui.horizontal_wrapped(|ui| {
        if action(
            ui,
            true,
            "\u{1F5C1} Open Program",
            "Open a program file, or a Johnny .ram file",
            OPEN,
        ) {
            state.open_program();
        }

        if action(
            ui,
            true,
            "\u{1F4BE} Save Program",
            "Save the memory and the instruction set to a file; name it .ram for a Johnny file",
            SAVE,
        ) {
            state.save_program();
        }

        if ui
            .button("Example")
            .on_hover_text("Load a small example program")
            .clicked()
        {
            state.load_example();
        }

        ui.separator();

        let halted = state.halted();

        let (label, hint) = if state.running {
            ("\u{23F8} Pause", "Pause the running program")
        } else {
            ("\u{25B6} Run", "Run the program at the chosen speed")
        };

        if action(ui, !halted, label, hint, RUN) {
            state.last_tick = Instant::now();
            state.running = !state.running;
        }

        if action(
            ui,
            !halted,
            "\u{23ED} Step",
            "Carry out one whole instruction",
            STEP,
        ) {
            state.step();
        }

        if action(
            ui,
            true,
            "\u{27F2} Reset",
            "Put the registers and the console back to the start; memory is kept",
            RESET,
        ) {
            state.reset();
        }

        // A menu, so that wiping the whole memory takes a second click.
        ui.menu_button("\u{1F5D1} Clear", |ui| {
            if ui.button("Erase all memory").clicked() {
                state.clear();
            }
        })
        .response
        .on_hover_text("Erase the whole memory");

        // The slider does not wrap by itself, and overflowing would widen the whole page.
        if ui.max_rect().right() - ui.cursor().left() < 240.0 {
            ui.end_row();
        } else {
            ui.separator();
        }

        ui.label("Speed");

        ui.add(
            egui::Slider::new(&mut state.speed, MIN_SPEED..=MAX_SPEED)
                .logarithmic(true)
                .suffix(" ips"),
        )
        .on_hover_text("Instructions per second while running");
    });
}
