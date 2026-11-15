pub mod controls;
pub mod debug_mode;
pub mod gui_state;
pub mod instruction_mode;
pub mod machine_mode;

use eframe::egui;
use std::time::Instant;

use crate::debugger::debug::Debug;
use gui_state::{AppState, MAX_SPEED, MIN_SPEED, UiMode};

const CONSOLE_PANEL_HEIGHT: f32 = 120.0;

// Remembered between launches: speed, the memory, registers, CPU, console and control unit
// toggles, then the advanced view and Johnny mode.
type Settings = (usize, bool, bool, bool, bool, bool, bool, bool);

pub struct VnmApp {
    pub state: AppState,
}

impl VnmApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();

        crate::gui::theme::apply(&cc.egui_ctx);

        if let Some((speed, memory, registers, cpu, console, control_unit, advanced, johnny)) = cc
            .storage
            .and_then(|storage| eframe::get_value::<Settings>(storage, eframe::APP_KEY))
        {
            state.speed = speed.clamp(MIN_SPEED, MAX_SPEED);
            state.show_memory = memory;
            state.show_registers = registers;
            state.show_cpu = cpu;
            state.show_console = console;
            state.show_control_unit = control_unit;
            state.advanced = advanced;

            if johnny {
                state.toggle_johnny();
                state.status_message.clear();
            }
        }

        Self { state }
    }
}

impl eframe::App for VnmApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = &self.state;
        let settings: Settings = (
            state.speed,
            state.show_memory,
            state.show_registers,
            state.show_cpu,
            state.show_console,
            state.show_control_unit,
            state.advanced,
            state.johnny(),
        );

        eframe::set_value(storage, eframe::APP_KEY, &settings);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_menu").show(ui, |ui| {
            ui.horizontal(|ui| {
                for (mode, label) in [
                    (UiMode::Machine, "Machine"),
                    (UiMode::Instructions, "Instruction Set"),
                    (UiMode::Guide, "Guide"),
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

                    // Framed, so that they read as switches and not as text.
                    if ui
                        .add(egui::Button::new("Johnny mode").selected(self.state.johnny()))
                        .on_hover_text(
                            "Work like the Johnny simulator: decimal words (two digits of \
                             instruction, three of address), 1000 cells and Johnny's instructions. \
                             Opens Johnny .ram and .mc files. Each mode keeps its own program",
                        )
                        .clicked()
                    {
                        self.state.toggle_johnny();
                    }

                    if ui
                        .add(egui::Button::new("Advanced").selected(self.state.advanced))
                        .on_hover_text(
                            "Show everything: the control unit with its micro steps, the buttons \
                             to drive the machine by hand and the view switches",
                        )
                        .clicked()
                    {
                        self.state.advanced = !self.state.advanced;
                    }
                });
            });
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let status = self.state.status();
                let color = match status {
                    "Running" => ui.visuals().selection.bg_fill,
                    "Halted" => ui.visuals().error_fg_color,
                    "Waiting for input" => ui.visuals().warn_fg_color,
                    _ => ui.visuals().text_color(),
                };

                ui.colored_label(color, egui::RichText::new(status).strong());
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

        if self.state.mode == UiMode::Machine && (self.state.show_console || !self.state.advanced) {
            egui::Panel::bottom("console_panel")
                .resizable(true)
                .default_size(CONSOLE_PANEL_HEIGHT)
                .min_size(CONSOLE_PANEL_HEIGHT)
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

            UiMode::Guide => {
                crate::gui::guide_view::show(ui, &self.state.machine);
            }

            UiMode::Debug => {
                let mut debug = self.state.debug.take().unwrap_or_else(Debug::new);
                debug_mode::show(ui, &mut self.state, &mut debug);
                self.state.debug = Some(debug);
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

            if self.state.machine.run_steps(steps) {
                self.state.running = false;
                let machine = &self.state.machine;

                self.state.status_message = format!(
                    "Breakpoint at {}",
                    machine.instruction_set.digits(machine.cpu.pc as i32, 4, 3)
                );
            }

            self.state.time_accumulator -= steps as f64 * secs_per_instruction;

            self.state.sync_running();

            ui.ctx().request_repaint();
        }
    }
}
