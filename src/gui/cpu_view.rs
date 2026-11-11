use egui::{Rect, Ui};

use super::micro_buttons;
use crate::machine::{ACC_MAX, ACC_MIN, Machine, MicroOp};

pub fn show(ui: &mut Ui, machine: &mut Machine) -> Option<Rect> {
    ui.heading("CPU");

    ui.horizontal(|ui| {
        ui.label("Program Counter");

        let mut pc = machine.cpu.pc;

        if ui
            .add(egui::DragValue::new(&mut pc))
            .on_hover_text("Address of the next instruction; drag or double-click to change it")
            .changed()
        {
            machine.cpu.pc = pc;
        }
    });

    if ui
        .button("Set entry point to PC")
        .on_hover_text("Reset will start the program at the current program counter")
        .clicked()
    {
        machine.entry_point = machine.cpu.pc;
    }

    ui.label(format!("Entry point: {}", machine.entry_point));

    micro_buttons(
        ui,
        machine,
        &[
            MicroOp::AccInc,
            MicroOp::AccDec,
            MicroOp::AccZero,
            MicroOp::Plus,
            MicroOp::Minus,
            MicroOp::Times,
            MicroOp::Divide,
            MicroOp::Input,
            MicroOp::Output,
        ],
    );

    let accumulator = ui
        .horizontal(|ui| {
            ui.label("Accumulator");

            let mut acc = machine.cpu.acc;
            let response = ui.add(egui::DragValue::new(&mut acc).range(ACC_MIN..=ACC_MAX));

            if response.changed() {
                machine.cpu.set_acc(acc);
            }

            // The zero lamp: said in words too, not only by colour.
            if machine.cpu.acc == 0 {
                ui.colored_label(ui.visuals().warn_fg_color, "= 0")
            } else {
                ui.weak("not 0")
            }
            .on_hover_text(
                "Whether the accumulator is zero, which is what TST, JZ and the =0: micro steps test",
            );

            response.rect
        })
        .inner;

    ui.clip_rect()
        .contains_rect(accumulator)
        .then_some(accumulator)
}
