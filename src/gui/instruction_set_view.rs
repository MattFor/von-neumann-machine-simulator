use egui::Ui;

use super::listing;
use crate::machine::{Machine, MicroOp, Opcode};

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
                    ui.label("Code").on_hover_text(
                        "The high byte of a memory word that selects this instruction",
                    );
                    ui.label("Mnemonic")
                        .on_hover_text("The name shown in the memory view");
                    ui.label("Operation").on_hover_text(
                        "What the instruction does; Custom lets you edit its micro steps",
                    );
                    ui.label("");
                    ui.label("");
                    ui.label("Micro steps")
                        .on_hover_text("What the control unit does, in order, after the fetch");
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

                        let before = definition.opcode;
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

                        // Custom starts from the steps of the operation it replaces.
                        if definition.opcode == Opcode::Custom && before != Opcode::Custom {
                            definition.micro = before.micro().to_vec();
                        }

                        if ui
                            .button("Remove")
                            .on_hover_text("Delete this instruction from the set")
                            .clicked()
                        {
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

                        ui.horizontal(|ui| {
                            if definition.opcode != Opcode::Custom {
                                ui.weak(listing(definition.steps()));
                                return;
                            }

                            let mut removed_step = None;

                            for (step, op) in definition.micro.iter().enumerate() {
                                let (label, hint) = op.info();

                                if ui
                                    .small_button(label)
                                    .on_hover_text(format!("{hint}\nClick to remove this step"))
                                    .clicked()
                                {
                                    removed_step = Some(step);
                                }
                            }

                            if let Some(step) = removed_step {
                                definition.micro.remove(step);
                            }

                            ui.menu_button("+", |ui| {
                                for op in MicroOp::ALL {
                                    let (label, hint) = op.info();

                                    if ui.button(label).on_hover_text(hint).clicked() {
                                        definition.micro.push(op);
                                    }
                                }
                            })
                            .response
                            .on_hover_text("Add a micro step at the end");
                        });

                        ui.end_row();
                    }
                });
        });

    if let Some(index) = removed {
        machine.instruction_set.definitions.remove(index);
    }
}
