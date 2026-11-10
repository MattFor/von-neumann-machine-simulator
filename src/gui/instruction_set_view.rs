use egui::Ui;

use crate::machine::Machine;
use crate::machine::Opcode;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.label("Name");

        ui.add(egui::TextEdit::singleline(&mut machine.instruction_set.name).desired_width(160.0));
    });

    ui.separator();

    let mut removed = None;
    let codes: Vec<i32> = machine
        .instruction_set
        .definitions
        .iter()
        .map(|definition| definition.code)
        .collect();

    egui::ScrollArea::vertical()
        .id_salt("instruction_set_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("instruction_set_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Code");
                    ui.label("Mnemonic");
                    ui.label("Operation");
                    ui.label("");
                    ui.end_row();

                    for (index, definition) in
                        machine.instruction_set.definitions.iter_mut().enumerate()
                    {
                        ui.add(
                            egui::DragValue::new(&mut definition.code)
                                .range(0..=255)
                                .hexadecimal(2, false, true),
                        );

                        ui.add(
                            egui::TextEdit::singleline(&mut definition.mnemonic)
                                .desired_width(110.0),
                        );

                        let operation_response = egui::ComboBox::from_id_salt(("operation", index))
                            .width(110.0)
                            .selected_text(format!("{:?}", definition.opcode))
                            .show_ui(ui, |ui| {
                                for candidate in Opcode::iter() {
                                    let response = ui.selectable_value(
                                        &mut definition.opcode,
                                        candidate,
                                        format!("{candidate:?}"),
                                    );

                                    response.on_hover_text(candidate.description());
                                }
                            });

                        operation_response
                            .response
                            .on_hover_text(definition.opcode.description());

                        if ui.button("Remove").clicked() {
                            removed = Some(index);
                        }

                        // Only the first definition of a code is ever decoded.
                        let duplicate = codes
                            .iter()
                            .filter(|code| **code == definition.code)
                            .count()
                            > 1;

                        // Always added: a label that comes and goes shifts widget ids and steals focus.
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            if duplicate { "duplicate code" } else { "" },
                        );

                        ui.end_row();
                    }
                });
        });

    if let Some(index) = removed {
        machine.instruction_set.definitions.remove(index);
    }
}
