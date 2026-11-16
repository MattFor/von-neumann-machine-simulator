pub mod controls;
pub mod debug_mode;
pub mod gui_state;
pub mod instruction_mode;
pub mod machine_mode;

use eframe::egui;
use std::time::{Duration, Instant};

use crate::debugger::debug::Debug;
use gui_state::{AppState, MAX_SPEED, MIN_SPEED, PRESETS, UiMode, preset_of};

const FRAME: Duration = Duration::from_micros(16_667);

const CUSTOM: (&str, &str) = (
    "Custom",
    "The instruction set or its format matches none of the machines; pick the machine again \
     to get its set back",
);

// Remembered between launches: speed, the memory, registers, CPU, console and control unit
// toggles, then the advanced view and whether the words are Johnny's, for older builds.
type Settings = (usize, bool, bool, bool, bool, bool, bool, bool);

// The picked machine, under a key of its own so that older settings still load.
const PRESET_KEY: &str = "preset";

pub struct VnmApp {
    pub state: AppState,
}

impl VnmApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();

        crate::gui::theme::apply(&cc.egui_ctx);

        let picked = cc
            .storage
            .and_then(|storage| eframe::get_value::<usize>(storage, PRESET_KEY));

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

            // Settings from before the picker only said whether it was Johnny mode, preset 1.
            if let Some(preset) = picked
                .or(johnny.then_some(1))
                .filter(|preset| *preset < PRESETS.len())
            {
                state.switch(preset);
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
            state.machine.instruction_set.format.johnny_compatible(),
        );

        eframe::set_value(storage, eframe::APP_KEY, &settings);
        eframe::set_value(storage, PRESET_KEY, &state.preset);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

impl VnmApp {
    fn draw(&mut self, ui: &mut egui::Ui) {
        // Before drawing, so the frame shows the instructions it ran.
        self.state.tick(Instant::now());

        let layout = self.state.layout();

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
                    egui::widgets::global_theme_preference_buttons(ui);

                    let current = preset_of(&self.state.machine.instruction_set);
                    let (label, hint) = current.map_or(CUSTOM, |preset| PRESETS[preset]);

                    egui::ComboBox::from_id_salt("machine_picker")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            for (preset, (name, hint)) in PRESETS.into_iter().enumerate() {
                                if ui
                                    .selectable_label(current == Some(preset), name)
                                    .on_hover_text(hint)
                                    .clicked()
                                {
                                    self.state.pick(preset);
                                }
                            }

                            if current.is_none() {
                                ui.selectable_label(true, CUSTOM.0).on_hover_text(CUSTOM.1);
                            }
                        })
                        .response
                        .on_hover_text(format!(
                            "The machine to simulate. {hint}. Each machine keeps its own program, \
                             so switching loses nothing"
                        ));

                    ui.label("Machine");

                    // Framed, so that it reads as a switch and not as text.
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

        // After drawing, so that Run pressed in this frame is seen. Fast runs catch up in tick, so
        // they need no more frames than a 60 Hz screen shows.
        if let Some(due) = self.state.due() {
            ui.ctx().request_repaint_after(due.max(FRAME));
        }

        // A toggle drawn after the panels it moves would leave them a frame behind, so the
        // pass is run again from the new state before anything is shown.
        if self.state.layout() != layout {
            ui.ctx().request_discard("the layout changed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // The repaint each frame asks for, over a few frames of no input a second apart, so that
    // egui's own animations end.
    fn repaints(app: &mut VnmApp, frames: usize, size: egui::Vec2) -> Vec<Duration> {
        let ctx = egui::Context::default();

        (0..frames)
            .map(|frame| {
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    time: Some(frame as f64),
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |ui| app.draw(ui));
                output.textures_delta.clear();

                output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
            })
            .collect()
    }

    // The smallest window, with every view and the console: the memory keeps three whole rows and
    // no text runs past the side of its card.
    #[test]
    fn the_smallest_window_shows_three_memory_rows_and_cuts_no_line_short() {
        let mut app = VnmApp {
            state: AppState::new(),
        };
        let ctx = egui::Context::default();
        crate::gui::theme::apply(&ctx);

        for preset in 0..PRESETS.len() {
            app.state.pick(preset);

            for mask in 0..64 {
                let state = &mut app.state;
                state.advanced = mask & 1 != 0;
                state.show_memory = mask & 2 != 0;
                state.show_registers = mask & 4 != 0;
                state.show_control_unit = mask & 8 != 0;
                state.show_cpu = mask & 16 != 0;
                state.show_console = mask & 32 != 0;

                let mut shapes = Vec::new();

                for frame in 0..3 {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(854.0, 480.0),
                        )),
                        time: Some(frame as f64),
                        ..Default::default()
                    };
                    let mut output = ctx.run_ui(input, |ui| app.draw(ui));

                    output.textures_delta.clear();
                    shapes = output.shapes;
                }

                let format = app.state.machine.instruction_set.format;
                let mut rows = Vec::new();

                for clipped in &shapes {
                    let egui::Shape::Text(text) = &clipped.shape else {
                        continue;
                    };

                    if text.galley.text().is_empty() {
                        continue;
                    }

                    let bounds = text.visual_bounding_rect();

                    assert!(
                        bounds.right() <= clipped.clip_rect.right() + 0.5,
                        "{preset} {mask} {:?}",
                        text.galley.text()
                    );

                    if clipped.clip_rect.contains_rect(bounds) {
                        rows.push(text.galley.text().to_string());
                    }
                }

                let memory = !app.state.advanced || app.state.show_memory;
                let shown = (0..3).all(|address| rows.contains(&format.operand_text(address)));

                assert!(!memory || shown, "{preset} {mask}");
            }
        }
    }

    #[test]
    fn a_paused_machine_asks_for_no_repaints_and_a_slow_one_for_one_per_instruction() {
        let mut app = VnmApp {
            state: AppState::new(),
        };

        let size = egui::vec2(1180.0, 760.0);

        for preset in 0..PRESETS.len() {
            app.state.pick(preset);

            for advanced in [false, true] {
                app.state.advanced = advanced;

                for size in [size, egui::vec2(700.0, 500.0)] {
                    let delays = repaints(&mut app, 6, size);

                    assert_eq!(delays[3..], [Duration::MAX; 3], "{preset} {advanced} {size:?}");
                }
            }
        }

        app.state.speed = 1;
        app.state.running = true;
        app.state.last_tick = Instant::now();

        let delays = repaints(&mut app, 6, size);

        assert!(
            delays[3..].iter().all(|delay| *delay > Duration::from_millis(500)),
            "{delays:?}"
        );

        // A fast one runs many instructions a frame, not a frame per instruction. egui's own
        // animations (the memory view following the PC) may still ask for one right away.
        app.state.speed = MAX_SPEED;

        let delays = repaints(&mut app, 6, size);

        assert!(delays[3..].iter().all(|delay| delay.is_zero() || *delay >= FRAME), "{delays:?}");
    }
}
