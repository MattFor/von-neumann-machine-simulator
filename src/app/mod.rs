pub mod controls;
pub mod debug_mode;
pub mod gui_state;
pub mod instruction_mode;
pub mod machine_mode;

use eframe::egui;
use std::time::Instant;

use gui_state::{AppState, UiMode};

const CONSOLE_PANEL_HEIGHT: f32 = 120.0;

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
                ui.label("View:");

                for (mode, label) in [
                    (UiMode::Machine, "Machine"),
                    (UiMode::Instructions, "Instruction Set"),
                ] {
                    if ui
                        .selectable_label(self.state.mode == mode, label)
                        .clicked()
                    {
                        self.state.mode = mode;
                    }
                }

                #[cfg(feature = "debug-mode")]
                if ui
                    .selectable_label(self.state.mode == UiMode::Debug, "Debug")
                    .clicked()
                {
                    self.state.mode = UiMode::Debug;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_switch(ui);
                });
            });
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(self.state.status());
                ui.separator();

                let message = &self.state.status_message;

                if message.starts_with("Could not") {
                    ui.colored_label(ui.visuals().error_fg_color, message);
                    ui.separator();
                } else if !message.is_empty() {
                    ui.label(message);
                    ui.separator();
                }

                ui.label(format!("Set: {}", self.state.machine.instruction_set.name));

                if let Some(name) = &self.state.program_name {
                    ui.separator();
                    ui.label(format!("Program: {name}"));
                }
            });
        });

        if self.state.mode == UiMode::Machine && self.state.show_console {
            egui::Panel::bottom("console_panel")
                .exact_size(CONSOLE_PANEL_HEIGHT)
                .show(ui, |ui| {
                    crate::gui::console_view::show(ui, &mut self.state.machine);
                });
        }

        egui::CentralPanel::default().show(ui, |ui| match self.state.mode {
            UiMode::Machine => {
                machine_mode::show(ui, &mut self.state);
            }

            UiMode::Instructions => {
                instruction_mode::show(ui, &mut self.state);
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

        if self.state.running && !self.state.machine.waiting {
            let now = Instant::now();
            // Clamped so a stall (file dialog, waiting for input) is not replayed as a burst.
            let dt = (now - self.state.last_tick).as_secs_f64().min(0.1);
            self.state.last_tick = now;

            self.state.time_accumulator += dt;
            let secs_per_instruction = 1.0 / self.state.speed as f64;
            let steps = (self.state.time_accumulator / secs_per_instruction).floor() as usize;

            self.state.machine.run_steps(steps);

            self.state.time_accumulator -= steps as f64 * secs_per_instruction;

            self.state.sync_running();

            ui.ctx().request_repaint();
        }
    }
}
