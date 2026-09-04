use egui::Ui;

use crate::machine::machine::Machine;
use crate::utilities::parse_value;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.heading("Console");

        if ui.button("Clear").clicked() {
            machine.output.clear();
        }

        if machine.waiting {
            ui.weak("waiting for input");
        }
    });

    egui::ScrollArea::vertical()
        .id_salt("console_scroll")
        .auto_shrink([false, false])
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
                .desired_width(120.0)
                .hint_text("input"),
        );

        let send = ui.button("Send").clicked()
            || (field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

        if send {
            for value in input.split_whitespace().filter_map(parse_value) {
                machine.push_input(value);
            }

            input.clear();
        }

        ui.data_mut(|data| data.insert_temp(id, input));
    });
}
