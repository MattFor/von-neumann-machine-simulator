use egui::Ui;

use crate::machine::{Format, Machine};

const CONSOLE_HEADER_AND_INPUT_HEIGHT: f32 = 56.0;

// Numbers as the command line's --input reads them, so 0xFFFF is -1 on MARIE.
fn parse(format: &Format, input: &str) -> Option<Vec<i32>> {
    input
        .split_whitespace()
        .map(|raw| format.parse_input(raw))
        .collect()
}

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.heading("Console");

        if ui
            .button("Clear")
            .on_hover_text("Clear the console output")
            .clicked()
        {
            machine.clear_output();
        }

        if machine.waiting {
            ui.colored_label(ui.visuals().warn_fg_color, "waiting for input");
        }
    });

    // Sticking to the bottom lands a frame late, so grown output gets one more frame, idle or not.
    let length = egui::Id::unique("console_length");
    let grew = ui.data_mut(|data| {
        std::mem::replace(data.get_temp_mut_or(length, 0), machine.output.len())
    }) != machine.output.len();

    if grew {
        ui.ctx().request_repaint();
    }

    let output_height = (ui.available_height() - CONSOLE_HEADER_AND_INPUT_HEIGHT).max(0.0);

    egui::ScrollArea::vertical()
        .id_salt("console_scroll")
        .auto_shrink([false, false])
        .max_height(output_height)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if machine.output.is_empty() {
                ui.weak("(no output)");
            } else {
                ui.monospace(&machine.output);
            }
        });

    ui.horizontal(|ui| {
        let id = egui::Id::unique("console_input");
        let mut input: String = ui.data(|data| data.get_temp(id)).unwrap_or_default();

        let field = ui.add(
            egui::TextEdit::singleline(&mut input)
                .desired_width(160.0)
                .hint_text("numbers, then Enter"),
        );

        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let values = parse(&machine.instruction_set.format, &input);
        let clicked = ui
            .add_enabled(values.is_some(), egui::Button::new("Send"))
            .on_hover_text("Queue these numbers for IN to read")
            .clicked();

        if enter {
            field.request_focus();
        }

        // Bad input stays in the field to be fixed instead of being dropped.
        match values {
            Some(values) if clicked || enter => {
                for value in values {
                    machine.push_input(value);
                }

                input.clear();
            }

            None => {
                ui.colored_label(ui.visuals().error_fg_color, "numbers only");
            }

            _ => {}
        }

        ui.data_mut(|data| data.insert_temp(id, input));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_input_is_read_in_the_machines_encoding() {
        assert_eq!(parse(&Format::marie(), "0xFFFF 12 -3"), Some(vec![-1, 12, -3]));
        assert_eq!(parse(&Format::default(), "0xFFFF"), Some(vec![0xffff]));
        assert_eq!(parse(&Format::marie(), "1 x"), None);
    }

    // A fast run that prints and halts left the console a frame short of its last lines.
    #[test]
    fn new_output_gets_a_frame_to_scroll_to_it() {
        let ctx = egui::Context::default();
        let mut machine = Machine::new();
        // A second apart, so that egui's own animations end.
        let frame = |machine: &mut Machine| {
            let input = egui::RawInput {
                time: Some(ctx.input(|input| input.time) + 1.0),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| show(ui, machine));
            output.textures_delta.clear();

            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
        };

        for _ in 0..3 {
            frame(&mut machine);
        }

        machine.print("1\n");

        let delays = [0; 4].map(|_| frame(&mut machine));

        assert_eq!(delays[0], std::time::Duration::ZERO);
        assert_eq!(delays[3], std::time::Duration::MAX);
    }
}
