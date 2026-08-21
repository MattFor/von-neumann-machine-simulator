use egui::Ui;

use crate::machine::machine::Machine;

const DISPLAYED_MEMORY_BLOCKS: usize = 255;
const CURRENT_BLOCK_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::from_rgb(55, 30, 0);
const DEFAULT_ADDRESS: &str = "0";

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    address_bus_panel(ui, machine);
    memory_grid_panel(ui, machine);
}

fn address_bus_panel(ui: &mut Ui, machine: &mut Machine) {
    ui.vertical(|ui| {
        let id = egui::Id::new("address_bus_edit");
        let mut address_bus: String = ui.data(|data| data.get_temp(id)).unwrap_or(DEFAULT_ADDRESS.to_string());
        let response = ui.add(egui::TextEdit::singleline(&mut address_bus));
        ui.data_mut(|data| data.insert_temp(id, address_bus.clone()));

        if ui.button("Set address bus").clicked() {
            if let Ok(parsed_address) = address_bus.parse::<i32>() {
                crate::machine::execute(
                    machine,
                    crate::machine::Instruction {
                        opcode: crate::machine::Opcode::Jump,
                        operand: parsed_address,
                    },
                );
            }
        }
    });
}

fn memory_grid_panel(ui: &mut Ui, machine: &mut Machine) {
    let pc = machine.cpu.pc as usize;

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("memory_grid")
            .striped(true)
            .with_row_color(move |row, _style| {
                // NOTE: pc incremented to avoid pointing to header
                // this is kinda bad design. In the future move header
                // out of the grid. For now I guess it's just fine.
                if row == (pc + 1) {
                    Some(CURRENT_BLOCK_HIGHLIGHT_COLOR)
                } else {
                    None
                }
            })
            .show(ui, |ui| {
                // NOTE: move header out of current grid!
                print_headers(ui);

                for row in 0..DISPLAYED_MEMORY_BLOCKS {
                    let raw = machine.memory.data()[row];
                    let instruction = crate::machine::decode(raw);
                    let opcode = instruction.opcode;
                    let operand = instruction.operand;

                    ui.label(format!("{row:02}"));

                    // Opcode display and edit
                    let mut selected_opcode = opcode;
                    egui::ComboBox::from_id_salt(("opcode", row))
                        .selected_text(format!("{:?}", selected_opcode as i32))
                        .show_ui(ui, |ui| {
                            for _opcode in crate::machine::Opcode::iter() {
                                let value = _opcode as i32;
                                let key = format!("{:?}", _opcode);

                                ui.selectable_value(&mut selected_opcode, _opcode, format!("{} - {}", value, key));
                            }
                        });

                    if selected_opcode != opcode {
                        update_memory_cell(machine, row, selected_opcode, operand);
                    }

                    // Operand display and edit
                    let id = ui.make_persistent_id(("operand_edit", row));
                    let mut operand_buffer = ui.data(|data| data.get_temp(id)).unwrap_or(operand.to_string());  
                    let response = ui.add(egui::TextEdit::singleline(&mut operand_buffer));
                    ui.data_mut(|data| data.insert_temp(id, operand_buffer.clone()));

                    // NOTE: check for UX. Right now does NOT
                    // require enter to confirm change
                    if response.lost_focus() {
                        if let Ok(parsed_operand) = operand_buffer.parse::<i32>() {
                            if parsed_operand != operand {
                                update_memory_cell(machine, row, opcode, parsed_operand);
                            }
                        }
                    }

                    ui.label(format!("{:?}", instruction.opcode));
                    ui.label(format!("{:?}", instruction.operand));

                    ui.end_row();
                }
        });
    });
}

fn update_memory_cell(machine: &mut Machine, address: usize, _opcode: crate::machine::Opcode, _operand: i32) {
    let instruction = crate::machine::Instruction {
        opcode: _opcode,
        operand: _operand,
    };

    let raw: i32 = crate::machine::encode(instruction);

    machine.memory.write(address, raw);
}

fn print_headers(ui: &mut Ui) {
    ui.label("Adr");
    ui.label("Hi");
    ui.label("Low");
    ui.label("Asm (Op)");
    ui.label("Opnd");
    ui.end_row();
}