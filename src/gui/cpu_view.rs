use egui::Ui;

use super::{micro_buttons, radix_drag, word_drag};
use crate::machine::{Machine, MicroOp};

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.heading("CPU");

    let format = machine.instruction_set.format;

    ui.horizontal_wrapped(|ui| {
        ui.label("Program Counter");

        let mut pc = machine.cpu.pc as i32;
        let last = format.cells() as i32 - 1;

        if ui
            .add(radix_drag(&mut pc, format, format.operand_digits).range(0..=last))
            .on_hover_text("Address of the next instruction; drag or double-click to change it")
            .changed()
        {
            machine.cpu.pc = pc as u16;
        }

        if ui
            .button("Set as entry point")
            .on_hover_text(format!(
                "Reset will start the program here instead of at {}",
                format.operand_text(machine.entry_point as i32)
            ))
            .clicked()
        {
            machine.entry_point = machine.cpu.pc;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Accumulator");

        let acc = machine.cpu.acc;
        let mut word = format.word(acc);

        if ui
            .add(word_drag(&mut word, format, format.acc_min..=format.acc_max))
            .on_hover_text(format!(
                "Where all arithmetic happens, {acc} in decimal; drag or double-click to change it"
            ))
            .changed()
        {
            machine.set_acc(format.value(word));
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
