pub mod controls;
pub mod debug_mode;
pub mod easy_mode;
pub mod gui_state;
pub mod hard_mode;

use eframe::egui;
use std::time::Instant;

use gui_state::{AppState, UiMode};

pub struct VnmApp {
    pub state: AppState,
}

impl VnmApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for VnmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_menu").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Mode:");

                if ui
                    .selectable_label(self.state.mode == UiMode::Easy, "Easy")
                    .clicked()
                {
                    self.state.mode = UiMode::Easy;
                }

                if ui
                    .selectable_label(self.state.mode == UiMode::Hard, "Hard")
                    .clicked()
                {
                    self.state.mode = UiMode::Hard;
                }

                #[cfg(feature = "debug-mode")]
                if ui
                    .selectable_label(self.state.mode == UiMode::Debug, "Debug")
                    .clicked()
                {
                    self.state.mode = UiMode::Debug;
                }
            });
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.state.status_message);

                if let Some(name) = &self.state.program_name {
                    ui.separator();
                    ui.label(format!("Program: {name}"));
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| match self.state.mode {
            UiMode::Easy => {
                easy_mode::show(ui, &mut self.state);
            }

            UiMode::Hard => {
                hard_mode::show(ui, &mut self.state);
            }

            UiMode::Debug => {
                if let Some(mut debug) = self.state.debug.take() {
                    debug_mode::show(ui, &mut self.state, &mut debug);
                    self.state.debug = Some(debug);
                } else {
                    let mut debug = crate::debugger::debug::Debug::new();
                    debug_mode::show(ui, &mut self.state, &mut debug);
                    self.state.debug = Some(debug);
                }
            }
        });

        if self.state.running {
            if self.state.halted() {
                self.state.running = false;
                self.state.status_message = "Halted".to_string();
            } else {
                let now = Instant::now();
                let dt = now - self.state.last_tick;
                self.state.last_tick = now;

                self.state.time_accumulator += dt.as_secs_f64();
                let secs_per_instruction = 1.0 / self.state.speed as f64;
                let steps = (self.state.time_accumulator / secs_per_instruction)
                    .floor()
                    .min(gui_state::MAX_SPEED as f64) as usize;

                self.state.machine.run_steps(steps);

                self.state.time_accumulator -= steps as f64 * secs_per_instruction;

                if self.state.halted() {
                    self.state.running = false;
                    self.state.status_message = "Halted".to_string();
                } else {
                    ui.ctx().request_repaint();
                }
            }
        }
    }
}
