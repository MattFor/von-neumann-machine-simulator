use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &Machine) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("memory_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.label("Adr");
                ui.label("Hi");
                ui.label("Low");
                ui.label("Asm (Dec)");
                ui.label("Opcode");
                ui.end_row();

                for row in 0..255 {
                    let raw = machine.memory.data()[row];
                    let instruction = crate::machine::decode(raw);
                    let opcode = (raw >> 8) & 0xff;
                    let operand = raw & 0xff;

                    ui.label(format!("{row:02X}"));
                    
                    let mut selected_opcode = opcode;
                    egui::ComboBox::from_id_salt(("opcode", row))
                        .selected_text(format!("{:?}", selected_opcode))
                        .show_ui(ui, |ui| {
                            // TODO
                            // for op in 0..10 { 
                            //     ui.selectable_value(&mut selected_opcode, op.to_string(), format!("{:02X} - UNFINISHED", op));
                            // }
                        });

                    let mut operand_value = format!("{:02X}", operand);
                    let response = ui.add(egui::TextEdit::singleline(&mut operand_value));
                    if response.changed() {
                        // TODO
                    }
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // TODO
                    }

                    ui.label(format!("{:?}", instruction.operand));
                    ui.label(format!("{:?}", instruction.opcode));

                    ui.end_row();
                }
            });
    });
}
