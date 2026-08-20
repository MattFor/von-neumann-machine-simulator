use egui::Ui;

use crate::machine::machine::Machine;

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("memory_grid").striped(true).show(ui, |ui| {
            ui.label("Adr");
            ui.label("Hi");
            ui.label("Low");
            ui.label("Asm (Dec)");
            ui.label("Opcode");
            ui.end_row();

            for row in 0..255 {
                let raw = machine.memory.data()[row];
                let instruction = crate::machine::decode(raw);
                let opcode = instruction.opcode;
                let operand = instruction.operand;

                ui.label(format!("{row:02}"));

                let mut selected_opcode = opcode;
                egui::ComboBox::from_id_salt(("opcode", row))
                    .selected_text(format!("{:?}", selected_opcode))
                    .show_ui(ui, |ui| {
                        for _opcode in crate::machine::Opcode::iter() {
                            let value = _opcode as i32;
                            let key = format!("{:?}", _opcode);

                            ui.selectable_value(&mut selected_opcode, _opcode, format!("{} - {}", value, key));
                        }
                    });

                if selected_opcode != opcode {
                    let _instruction = crate::machine::Instruction {
                        opcode: selected_opcode,
                        operand: operand,
                    };

                    let _raw: i32 = crate::machine::encode(_instruction);

                    machine.memory.write(row, _raw);
                }


                let id = ui.make_persistent_id(("operand_edit", row));

                let mut operand_buffer = ui.data(|data| data.get_temp(id)).unwrap_or(operand.to_string());  

                let response = ui.add(egui::TextEdit::singleline(&mut operand_buffer));

                ui.data_mut(|data| data.insert_temp(id, operand_buffer.clone()));

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(parsed_operand) = operand_buffer.parse::<i32>() {
                        let _instruction = crate::machine::Instruction {
                            opcode: selected_opcode,
                            operand: parsed_operand,
                        };

                        let _raw: i32 = crate::machine::encode(_instruction);

                        machine.memory.write(row, _raw);
                    }
                }
                

                ui.label(format!("{:?}", instruction.operand));
                ui.label(format!("{:?}", instruction.opcode));

                ui.end_row();
            }
        });
    });
}
