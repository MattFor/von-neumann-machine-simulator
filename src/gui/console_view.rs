use egui::Ui;

use crate::machine::Machine;
use crate::utilities::parse_value;

const CONSOLE_HEADER_AND_INPUT_HEIGHT: f32 = 56.0;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.heading("Console");

        if ui
            .button("Clear")
            .on_hover_text("Clear the console output")
            .clicked()
        {
            machine.output.clear();
        }

        if machine.waiting {
            ui.colored_label(ui.visuals().warn_fg_color, "waiting for input");
        }
    });

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
        let id = egui::Id::new("console_input");
        let mut input: String = ui.data(|data| data.get_temp(id)).unwrap_or_default();

        let field = ui.add(
            egui::TextEdit::singleline(&mut input)
                .desired_width(160.0)
                .hint_text("numbers, then Enter"),
        );

        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let values: Option<Vec<i32>> = input.split_whitespace().map(parse_value).collect();
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
