pub mod gui_state;
pub mod easy_mode;
pub mod hard_mode;

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
            });
        });

        egui::CentralPanel::default().show(ui, |ui| match self.state.mode {
            UiMode::Easy => {
                easy_mode::show(ui, &mut self.state);
            }

            UiMode::Hard => {
                hard_mode::show(ui, &mut self.state);
            }
        });

        if self.state.running {
            self.state.machine.step();

            ui.ctx().request_repaint();
        }
    }
}
