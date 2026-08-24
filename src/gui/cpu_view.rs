use egui::Ui;

use crate::machine::machine::Machine;
use crate::machine::{ACC_MAX, ACC_MIN, Instruction, Opcode, execute};

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.heading("CPU");

    ui.horizontal(|ui| {
        if ui.button("acc++").clicked() {
            execute(
                machine,
                Instruction {
                    opcode: Opcode::Add,
                    operand: 1,
                },
            );
        }

        if ui.button("acc--").clicked() {
            execute(
                machine,
                Instruction {
                    opcode: Opcode::Sub,
                    operand: 1,
                },
            );
        }

        if ui.button("acc:=0").clicked() {
            machine.cpu.set_acc(0);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Accumulator");

        let mut acc = machine.cpu.acc;

        if ui
            .add(egui::DragValue::new(&mut acc).range(ACC_MIN..=ACC_MAX))
            .changed()
        {
            machine.cpu.set_acc(acc);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Program Counter");

        let mut pc = machine.cpu.pc;

        if ui.add(egui::DragValue::new(&mut pc)).changed() {
            machine.cpu.pc = pc;
        }
    });

    if ui.button("Set entry point to PC").clicked() {
        machine.entry_point = machine.cpu.pc;
    }

    ui.label(format!("Entry point: {}", machine.entry_point));
}
