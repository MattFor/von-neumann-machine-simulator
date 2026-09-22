use egui::{Rect, Ui};

use crate::machine::machine::Machine;
use crate::machine::{ACC_MAX, ACC_MIN};

pub fn show(ui: &mut Ui, machine: &mut Machine) -> Option<Rect> {
    ui.heading("CPU");

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

    ui.horizontal(|ui| {
        if ui.button("acc++").clicked() {
            machine.cpu.set_acc(machine.cpu.acc.saturating_add(1));
        }

        if ui.button("acc--").clicked() {
            machine.cpu.set_acc(machine.cpu.acc.saturating_sub(1));
        }

        if ui.button("acc:=0").clicked() {
            machine.cpu.set_acc(0);
        }
    });

    let accumulator = ui
        .horizontal(|ui| {
            ui.label("Accumulator");

            let mut acc = machine.cpu.acc;
            let response = ui.add(egui::DragValue::new(&mut acc).range(ACC_MIN..=ACC_MAX));

            if response.changed() {
                machine.cpu.set_acc(acc);
            }

            response.rect
        })
        .inner;

    ui.clip_rect()
        .contains_rect(accumulator)
        .then_some(accumulator)
}
