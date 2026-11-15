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

    micro_buttons(ui, machine, &BUTTONS);
}

// The arithmetic and console micro steps; the ones only some machines use show with their set.
pub const BUTTONS: [MicroOp; 12] = [
    MicroOp::AccInc,
    MicroOp::AccDec,
    MicroOp::AccZero,
    MicroOp::Plus,
    MicroOp::Minus,
    MicroOp::Times,
    MicroOp::Divide,
    MicroOp::Shift,
    MicroOp::Input,
    MicroOp::InputToDb,
    MicroOp::Output,
    MicroOp::DbToOut,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{DataEncoding, InstructionSet};

    // A smaller format leaves registers outside it until the machine next touches them.
    #[test]
    fn drawing_never_changes_a_register_the_format_no_longer_holds() {
        let ctx = egui::Context::default();
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::lmc();
        machine.cpu.pc = 150;
        machine.cpu.acc = -5;
        machine.cpu.mbr = -7;
        machine.instruction_set.format.acc_min = 0;
        machine.instruction_set.format.data_encoding = DataEncoding::Unsigned;

        for _ in 0..2 {
            ctx.run_ui(egui::RawInput::default(), |ui| {
                show(ui, &mut machine);
                super::super::data_bus_view::show(ui, ui.max_rect(), &mut machine, None, None, true);
            })
            .textures_delta
            .clear();
        }

        assert_eq!((machine.cpu.pc, machine.cpu.acc, machine.cpu.mbr), (150, -5, -7));
    }
}
