pub mod console_view;
pub mod control_unit_view;
pub mod cpu_view;
pub mod data_bus_view;
pub mod guide_view;
pub mod instruction_set_view;
pub mod memory_view;
pub mod register_view;
pub mod theme;

use egui::{KeyboardShortcut, Ui};

use crate::machine::{Format, Machine, MicroOp};

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

pub fn listing(steps: &[MicroOp]) -> String {
    let labels: Vec<&str> = steps.iter().map(|op| op.info().0).collect();

    labels.join("  ")
}
