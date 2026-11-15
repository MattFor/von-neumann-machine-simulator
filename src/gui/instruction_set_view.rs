use egui::Ui;

use super::{listing, radix_drag};
use crate::machine::{Definition, InstructionSet, Machine, MicroOp, Opcode};

// Whether two definitions are tried for the same words at the same step of decoding, so that the
// later one loses them. A fixed operand is tried before a mask, and both before none at all.
fn overlap(a: &Definition, b: &Definition, max_operand: i32) -> bool {
    let masked = |definition: &Definition| definition.operand_mask.zip(definition.operand);

    a.code == b.code
        && match (masked(a), masked(b)) {
            (Some((mask_a, a)), Some((mask_b, b))) => (0..=max_operand)
                .any(|operand| operand & mask_a == a & mask_a && operand & mask_b == b & mask_b),
            (None, None) => a.operand == b.operand,
            _ => false,
        }
}

// For each definition, whether another one takes some of its words.
pub fn duplicates(set: &InstructionSet) -> Vec<bool> {
    let max_operand = set.limits().1;
    let definitions = &set.definitions;

    (0..definitions.len())
        .map(|index| {
            (0..definitions.len()).any(|other| {
                other != index && overlap(&definitions[index], &definitions[other], max_operand)
            })
        })
        .collect()
}

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        ui.label("Name");

        ui.add(egui::TextEdit::singleline(&mut machine.instruction_set.name).desired_width(160.0));
    });

    // Such a set would not load back, nor save with a program.
    if let Err(error) = machine.instruction_set.check() {
        ui.colored_label(ui.visuals().error_fg_color, error.to_string());
    }

    ui.separator();

    let mut removed = None;
    let duplicates = duplicates(&machine.instruction_set);

    egui::ScrollArea::vertical()
        .id_salt("instruction_set_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("instruction_set_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Code").on_hover_text(
                        "The code part of a memory word that selects this instruction",
                    );
                    ui.label("Operand").on_hover_text(
                        "Ticked, the operand is part of the instruction too, like LMC's INP (901) \
                         and OUT (902); picking it in the memory writes this operand",
                    );
                    ui.label("Mask").on_hover_text(
                        "Ticked, only the operand bits under the mask have to match, like MARIE's \
                         Skipcond, which only looks at the top two bits",
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

                    let format = machine.instruction_set.format;
                    let (max_code, max_operand, _) = machine.instruction_set.limits();

                    for (index, definition) in
                        machine.instruction_set.definitions.iter_mut().enumerate()
                    {
                        ui.add(
                            radix_drag(&mut definition.code, format, format.code_digits)
                                .range(0..=max_code),
                        );

                        // Always added, only greyed out: fields that come and go shift widget ids.
                        let mut operand = definition.operand.unwrap_or(0);
                        let mut fixed = definition.operand.is_some();

                        ui.horizontal(|ui| {
                            ui.add(egui::Checkbox::without_text(&mut fixed))
                                .on_hover_text("Make the operand part of this instruction");
                            ui.add_enabled(
                                fixed,
                                radix_drag(&mut operand, format, format.operand_digits)
                                    .range(0..=max_operand),
                            );
                        });

                        definition.operand = fixed.then_some(operand);

                        // A mask needs a fixed operand to compare with.
                        let mut mask = definition.operand_mask.unwrap_or(max_operand);
                        let mut masked = fixed && definition.operand_mask.is_some();

                        ui.horizontal(|ui| {
                            ui.add_enabled(fixed, egui::Checkbox::without_text(&mut masked))
                                .on_hover_text("Compare only the operand bits under the mask");
                            ui.add_enabled(
                                masked,
                                radix_drag(&mut mask, format, format.operand_digits)
                                    .range(0..=max_operand),
                            );
                        });

                        definition.operand_mask = masked.then_some(mask);

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

                        // Always added: a label that comes and goes shifts widget ids and steals focus.
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            if duplicates[index] { "duplicate" } else { "" },
                        )
                        .on_hover_text(
                            "Another instruction matches the same words, and only the first one \
                             in the list runs for them",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn with(mut set: InstructionSet, definition: Definition) -> Vec<bool> {
        set.definitions.push(definition);

        duplicates(&set)
    }

    #[test]
    fn only_definitions_that_match_the_same_words_are_duplicates() {
        for preset in InstructionSet::presets() {
            assert!(!duplicates(&preset).contains(&true), "{}", preset.name);
        }

        // A second ADD with no fixed operand, then one for every code 9 word but INP's and OUT's.
        let add = with(InstructionSet::lmc(), Definition {
            code: 1,
            ..Default::default()
        });
        let catch_all = with(InstructionSet::lmc(), Definition {
            code: 9,
            ..Default::default()
        });

        assert_eq!(add.iter().filter(|duplicate| **duplicate).count(), 2);
        assert!(!catch_all.contains(&true));

        // Bit 0x400 set: what Skipcond =0 matches too, but not <0 or >0.
        let marie = InstructionSet::marie();
        let skipcond = with(marie.clone(), Definition {
            code: 8,
            operand: Some(0x400),
            operand_mask: Some(0x400),
            ..Default::default()
        });
        let flagged: Vec<&str> = marie
            .definitions
            .iter()
            .map(|definition| definition.mnemonic.as_str())
            .chain([""])
            .zip(skipcond)
            .filter(|(_, duplicate)| *duplicate)
            .map(|(mnemonic, _)| mnemonic)
            .collect();

        assert_eq!(flagged, ["Skipcond =0", ""]);
    }
}
