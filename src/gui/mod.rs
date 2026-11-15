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

use crate::machine::{DataEncoding, Format, Machine, MicroOp};

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

// Buttons that fire micro steps by hand.
pub fn micro_buttons(ui: &mut Ui, machine: &mut Machine, ops: &[MicroOp]) {
    ui.horizontal_wrapped(|ui| {
        for &op in ops {
            let (label, hint) = op.info();

            if ui.button(label).on_hover_text(hint).clicked() {
                machine.manual(op);
            }
        }
    });
}

// A number field that shows and reads the radix of the format, padded to `width` digits.
pub fn radix_drag(value: &mut i32, format: Format, width: u32) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .custom_formatter(move |number, _| format.text(number as i32, width))
        .custom_parser(move |text| format.parse(text).map(f64::from))
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
