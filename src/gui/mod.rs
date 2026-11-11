pub mod console_view;
pub mod control_unit_view;
pub mod cpu_view;
pub mod data_bus_view;
pub mod instruction_set_view;
pub mod memory_view;
pub mod register_view;

use egui::{KeyboardShortcut, Ui};

use crate::machine::{Machine, MicroOp};

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

pub fn listing(steps: &[MicroOp]) -> String {
    let labels: Vec<&str> = steps.iter().map(|op| op.info().0).collect();

    labels.join("  ")
}
