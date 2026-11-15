use egui::Ui;

use super::micro_buttons;
use crate::machine::{ACC_MAX, ACC_MIN, Machine, MicroOp};

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.heading("CPU");

    ui.horizontal_wrapped(|ui| {
        ui.label("Program Counter");

        let mut pc = machine.cpu.pc;

        if ui
            .add(egui::DragValue::new(&mut pc))
            .on_hover_text("Address of the next instruction; drag or double-click to change it")
            .changed()
        {
            machine.cpu.pc = pc;
        }

        if ui
            .button("Set as entry point")
            .on_hover_text(format!(
                "Reset will start the program here instead of at {}",
                machine.entry_point
            ))
            .clicked()
        {
            machine.entry_point = machine.cpu.pc;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Accumulator");

        let mut acc = machine.cpu.acc;

        if ui
            .add(egui::DragValue::new(&mut acc).range(ACC_MIN..=ACC_MAX))
            .on_hover_text("Where all arithmetic happens; drag or double-click to change it")
            .changed()
        {
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
    });

    ui.separator();

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
}
