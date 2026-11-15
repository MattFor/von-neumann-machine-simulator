use egui::Ui;

use crate::machine::{DataEncoding, Format, InstructionSet, PcOverflow, UnknownCode};

// Only a format the whole set still checks out with is applied, so the machine never gets one
// its registers cannot hold or that leaves an instruction without its code.
pub fn apply(set: &mut InstructionSet, format: Format) -> anyhow::Result<()> {
    if format != set.format {
        InstructionSet {
            format,
            ..set.clone()
        }
        .check()?;

        set.format = format;
    }

    Ok(())
}

// A count that can also be as many as the digits reach.
fn limit(ui: &mut Ui, label: &str, hint: &str, value: &mut Option<u32>, span: u32) {
    let mut all = value.is_none();
    let mut count = value.unwrap_or(span);

    ui.label(label).on_hover_text(hint);
    ui.checkbox(&mut all, "all")
        .on_hover_text("As many as the digits reach");
    ui.add_enabled(!all, egui::DragValue::new(&mut count).range(1..=65_536))
        .on_hover_text(hint);

    *value = (!all).then_some(count);
}

fn choice<T: PartialEq + Copy>(ui: &mut Ui, label: &str, value: &mut T, options: &[(T, &str, &str)]) {
    let selected = options
        .iter()
        .find(|(option, ..)| option == value)
        .map_or("", |(_, name, _)| name);

    ui.label(label);

    egui::ComboBox::from_id_salt(label)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for &(option, name, hint) in options {
                ui.selectable_value(value, option, name).on_hover_text(hint);
            }
        });
}

pub fn show(ui: &mut Ui, set: &mut InstructionSet) {
    let id = egui::Id::unique("format_draft");

    // A draft is kept while it is invalid, and dropped once the set changes some other way, so
    // it never lands on another machine's set.
    let mut draft = ui
        .data(|data| data.get_temp::<(InstructionSet, Format)>(id))
        .filter(|(drafted_for, _)| drafted_for == set)
        .map_or(set.format, |(_, draft)| draft);

    egui::CollapsingHeader::new("Format")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (label, hint, value, range) in [
                    (
                        "Radix",
                        "The base words, codes and operands are written in: 16 is hex, 10 decimal",
                        &mut draft.radix,
                        2..=16,
                    ),
                    (
                        "Code digits",
                        "How many digits of a word are the instruction code",
                        &mut draft.code_digits,
                        1..=16,
                    ),
                    (
                        "Operand digits",
                        "How many digits of a word are the operand, usually an address",
                        &mut draft.operand_digits,
                        1..=16,
                    ),
                ] {
                    ui.label(label).on_hover_text(hint);
                    ui.add(egui::DragValue::new(value).range(range))
                        .on_hover_text(hint);
                }

                let (codes, cells) = (draft.span(draft.code_digits), draft.span(draft.operand_digits));

                limit(ui, "Codes", "How many instruction codes there are", &mut draft.codes, codes);
                limit(ui, "Cells", "How many memory cells there are", &mut draft.cells, cells);
            });

            ui.horizontal_wrapped(|ui| {
                let hint = "The smallest and the largest number the accumulator holds";

                ui.label("Accumulator from").on_hover_text(hint);
                ui.add(egui::DragValue::new(&mut draft.acc_min))
                    .on_hover_text(hint);
                ui.label("to");
                ui.add(egui::DragValue::new(&mut draft.acc_max))
                    .on_hover_text(hint);

                choice(
                    ui,
                    "Negative numbers",
                    &mut draft.data_encoding,
                    &[
                        (
                            DataEncoding::Unsigned,
                            "Not in memory",
                            "Memory holds no negative numbers; a negative accumulator is stored as 0",
                        ),
                        (
                            DataEncoding::TwosComplement,
                            "Two's complement",
                            "Numbers are 16-bit two's complement, so -1 is stored as FFFF, like on MARIE",
                        ),
                        (
                            DataEncoding::Lmc,
                            "Signed cells",
                            "A cell keeps its sign, from -999 to 999, like on the Little Man Computer",
                        ),
                    ],
                );
            });

            ui.horizontal_wrapped(|ui| {
                choice(
                    ui,
                    "Unknown code",
                    &mut draft.unknown_code,
                    &[
                        (UnknownCode::Nop, "Skip it", "A word whose code is no instruction does nothing"),
                        (UnknownCode::Halt, "Halt", "A word whose code is no instruction halts the machine"),
                    ],
                );

                choice(
                    ui,
                    "After the last cell",
                    &mut draft.pc_overflow,
                    &[
                        (PcOverflow::Wrap, "Back to 0", "The program counter goes back to the first cell"),
                        (PcOverflow::Stay, "Stay there", "The program counter stays on the last cell, like in Johnny"),
                        (PcOverflow::Halt, "Halt", "The machine halts"),
                    ],
                );

                ui.checkbox(&mut draft.stop_when_pc_stays, "Stop when the PC stays")
                    .on_hover_text(
                        "Run stops on an instruction that leaves the program counter where it was, \
                         like a jump to itself",
                    );
            });

            if let Err(error) = apply(set, draft) {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(ui.visuals().error_fg_color, format!("Not applied: {error}"));

                    if ui
                        .button("Revert")
                        .on_hover_text("Go back to the format in use")
                        .clicked()
                    {
                        draft = set.format;
                    }
                });
            }
        });

    ui.data_mut(|data| data.insert_temp(id, (set.clone(), draft)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invalid_format_is_never_applied() {
        let mut set = InstructionSet::default();
        let wide = Format {
            operand_digits: 3,
            ..set.format
        };
        // Valid by itself, but HALT's code 255 would be gone.
        let fewer_codes = Format {
            codes: Some(16),
            ..set.format
        };

        assert!(apply(&mut set, wide).is_err());
        assert!(fewer_codes.check().is_ok());
        assert!(
            apply(&mut set, fewer_codes)
                .unwrap_err()
                .to_string()
                .contains("codes must be between")
        );
        assert_eq!(set.format, Format::default());

        let smaller = Format {
            cells: Some(16),
            acc_min: -100,
            ..set.format
        };

        apply(&mut set, smaller).unwrap();

        assert_eq!(set.format, smaller);
    }
}
