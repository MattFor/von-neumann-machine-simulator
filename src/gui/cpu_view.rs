use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        if ui.button("acc++").clicked() {
            crate::machine::execute(
                machine, 
                crate::machine::Instruction {
                    opcode: crate::machine::Opcode::Add,
                    operand: 1,
                }
            );
        }

        if ui.button("acc--").clicked() {
            crate::machine::execute(
                machine, 
                crate::machine::Instruction {
                    opcode: crate::machine::Opcode::Sub,
                    operand: 1,
                }
            );
        }

        if ui.button("acc:=0").clicked() {
            machine.cpu.acc = 0; // Doesnt match any opcodes
        }
    });

    ui.label(format!("Accumulator: {}", machine.cpu.acc));
}