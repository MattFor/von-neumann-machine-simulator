pub mod controls;
pub mod debug_mode;
pub mod gui_state;
pub mod instruction_mode;
pub mod machine_mode;

use eframe::egui;

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
            });
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.state.status_message);

                ui.separator();
                ui.label(format!("Set: {}", self.state.machine.instruction_set.name));

                if let Some(name) = &self.state.program_name {
                    ui.separator();
                    ui.label(format!("Program: {name}"));
                }
            });
        });

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

        if self.state.running {
            self.state.machine.run_steps(self.state.speed);
            self.state.sync_running();

            if self.state.running {
                ui.ctx().request_repaint();
            }
        }
    }
}
