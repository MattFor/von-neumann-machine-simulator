use egui::Ui;

use crate::machine::Opcode;
use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.label("Name");

        ui.add(egui::TextEdit::singleline(&mut machine.instruction_set.name).desired_width(160.0));
    });

    ui.separator();

    let mut removed = None;

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

                        egui::ComboBox::from_id_salt(("operation", index))
                            .width(110.0)
                            .selected_text(format!("{:?}", definition.opcode))
                            .show_ui(ui, |ui| {
                                for candidate in Opcode::iter() {
                                    ui.selectable_value(
                                        &mut definition.opcode,
                                        candidate,
                                        format!("{candidate:?}"),
                                    );
                                }
                            });

                        if ui.button("Remove").clicked() {
                            removed = Some(index);
                        }

                        ui.end_row();
                    }
                });
        });

    if let Some(index) = removed {
        machine.instruction_set.definitions.remove(index);
    }
}
