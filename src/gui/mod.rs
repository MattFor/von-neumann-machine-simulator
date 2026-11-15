pub mod console_view;
pub mod control_unit_view;
pub mod cpu_view;
pub mod data_bus_view;
pub mod format_view;
pub mod guide_view;
pub mod instruction_set_view;
pub mod memory_view;
pub mod register_view;
pub mod theme;

use std::ops::RangeInclusive;

use egui::{KeyboardShortcut, Ui};

use crate::machine::{DataEncoding, Format, InstructionSet, Machine, MicroOp};

// A button that also fires on its keyboard shortcut.
pub fn action(
    ui: &mut Ui,
    enabled: bool,
    label: &str,
    hint: &str,
    shortcut: KeyboardShortcut,
) -> bool {
    let pressed = enabled && ui.input_mut(|input| input.consume_shortcut(&shortcut));
    let hint = format!("{hint} ({})", ui.ctx().format_shortcut(&shortcut));

    ui.add_enabled(enabled, egui::Button::new(label))
        .on_hover_text(&hint)
        .on_disabled_hover_text(&hint)
        .clicked()
        || pressed
}

// Micro steps only some machines use; they get a button when the loaded set has them.
const OCCASIONAL: [MicroOp; 14] = [
    MicroOp::InsToPcIfNonNegative,
    MicroOp::InsToPcIfNegative,
    MicroOp::InsToPcIfCarry,
    MicroOp::InsToPcIfZeroFlag,
    MicroOp::PcToDb,
    MicroOp::ReturnToDb,
    MicroOp::PcIncIfPositive,
    MicroOp::PcIncIfNegative,
    MicroOp::DbToAb,
    MicroOp::AbToPc,
    MicroOp::AbLast,
    MicroOp::Shift,
    MicroOp::InputToDb,
    MicroOp::DbToOut,
];

// The micro steps of `ops` that get a button: the usual ones, and the others the set uses.
pub fn hand_ops(set: &InstructionSet, ops: &[MicroOp]) -> Vec<MicroOp> {
    ops.iter()
        .copied()
        .filter(|op| {
            !OCCASIONAL.contains(op)
                || set
                    .definitions
                    .iter()
                    .any(|definition| definition.steps().contains(op))
        })
        .collect()
}

// Buttons that fire micro steps by hand.
pub fn micro_buttons(ui: &mut Ui, machine: &mut Machine, ops: &[MicroOp]) {
    ui.horizontal_wrapped(|ui| {
        for op in hand_ops(&machine.instruction_set, ops) {
            let (label, hint) = op.info();

            if ui.button(label).on_hover_text(hint).clicked() {
                machine.manual(op);
            }
        }
    });
}

// A number field that shows and reads the radix of the format, padded to `width` digits.
// Its range only limits edits, so that drawing a value a smaller format left behind keeps it.
pub fn radix_drag(value: &mut i32, format: Format, width: u32) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .custom_formatter(move |number, _| format.text(number as i32, width))
        .custom_parser(move |text| format.parse(text).map(f64::from))
        .clamp_existing_to_range(false)
}

// A field for a number as the machine stores it: a whole word in its radix, so on MARIE FFFF is
// typed for -1. `word` is format.word() of the number; format.value() of it is the new number.
pub fn word_drag(word: &mut i32, format: Format, values: RangeInclusive<i32>) -> egui::DragValue<'_> {
    let words = match format.data_encoding {
        DataEncoding::TwosComplement => 0..=0xffff,
        DataEncoding::Unsigned | DataEncoding::Lmc => values,
    };

    radix_drag(word, format, format.code_digits + format.operand_digits)
        .range(words)
        // -1 can be typed as well as FFFF.
        .custom_parser(move |text| format.parse(text).map(|number| f64::from(format.word(number))))
}

pub fn listing(steps: &[MicroOp]) -> String {
    let labels: Vec<&str> = steps.iter().map(|op| op.info().0).collect();

    labels.join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_micro_step_of_the_loaded_set_has_one_button_and_the_others_none() {
        let bus = [MicroOp::RamToDb, MicroOp::DbToRam, MicroOp::AccToDb, MicroOp::DbToAcc];
        let all = [&cpu_view::BUTTONS[..], &control_unit_view::BUTTONS[..], &bus[..]].concat();

        for op in MicroOp::ALL {
            assert_eq!(all.iter().filter(|other| **other == op).count(), 1, "{op:?}");
        }

        for set in InstructionSet::presets() {
            let shown = [
                hand_ops(&set, &cpu_view::BUTTONS),
                hand_ops(&set, &control_unit_view::BUTTONS),
                bus.to_vec(),
            ]
            .concat();

            for definition in &set.definitions {
                for op in definition.steps() {
                    assert!(shown.contains(op), "{} {op:?}", set.name);
                }
            }
        }

        let standard = InstructionSet::default();

        assert!(!hand_ops(&standard, &MicroOp::ALL).contains(&MicroOp::Shift));
        assert!(hand_ops(&InstructionSet::cardiac(), &MicroOp::ALL).contains(&MicroOp::Shift));
        assert!(hand_ops(&InstructionSet::sap1(), &MicroOp::ALL).contains(&MicroOp::InsToPcIfCarry));
    }
}
