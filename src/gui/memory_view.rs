use egui::Ui;

use crate::machine::machine::Machine;
use crate::machine::{Instruction, Opcode, encode, execute};
use crate::utilities::parse_value;

const VISIBLE_CELLS: usize = 256;
const CURRENT_BLOCK_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::from_rgb(55, 30, 0);
const DEFAULT_ADDRESS: &str = "0";

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.heading("Memory");

    address_bus_panel(ui, machine);

    ui.separator();

    memory_grid_panel(ui, machine);
}

fn address_bus_panel(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        let id = egui::Id::new("address_bus_edit");
        let mut address_bus: String = ui
            .data(|data| data.get_temp(id))
            .unwrap_or(DEFAULT_ADDRESS.to_string());

        ui.add(egui::TextEdit::singleline(&mut address_bus).desired_width(80.0));
        ui.data_mut(|data| data.insert_temp(id, address_bus.clone()));

        if ui.button("Set address bus").clicked() {
            if let Some(parsed_address) = parse_value(&address_bus) {
                execute(
                    machine,
                    Instruction {
                        opcode: Opcode::Jump,
                        operand: parsed_address,
                    },
                );
            }
        }
    });
}

fn memory_grid_panel(ui: &mut Ui, machine: &mut Machine) {
    let pc = machine.cpu.pc as usize;
    let row_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    let definitions = machine.instruction_set.definitions.clone();

    egui::ScrollArea::vertical()
        .id_salt("memory_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, row_height, VISIBLE_CELLS, |ui, rows| {
            // NOTE: only the visible slice is rendered, so grid row indices are
            // relative to `first_address`.
            let first_address = rows.start;

            egui::Grid::new("memory_grid")
                .striped(true)
                .with_row_color(move |row, _style| {
                    // NOTE: pc offset by one to avoid pointing to the header
                    // this is kinda bad design. In the future move header
                    // out of the grid. For now I guess it's just fine.
                    if pc >= first_address && row == (pc - first_address + 1) {
                        Some(CURRENT_BLOCK_HIGHLIGHT_COLOR)
                    } else {
                        None
                    }
                })
                .show(ui, |ui| {
                    // NOTE: move header out of current grid!
                    print_headers(ui);

                    for address in rows {
                        let raw = machine.memory.read(address);

                        let mut code = (raw >> 8) & 0xff;
                        let mut operand = raw & 0xff;

                        ui.monospace(format!("{address:04X}"));

                        egui::ComboBox::from_id_salt(("opcode", address))
                            .width(110.0)
                            .selected_text(machine.instruction_set.mnemonic(code))
                            .show_ui(ui, |ui| {
                                for definition in &definitions {
                                    ui.selectable_value(
                                        &mut code,
                                        definition.code,
                                        format!(
                                            "{:02X} - {}",
                                            definition.code, definition.mnemonic
                                        ),
                                    );
                                }
                            });

                        ui.add(
                            egui::DragValue::new(&mut operand)
                                .range(0..=255)
                                .hexadecimal(2, false, true),
                        );

                        let updated = encode(code, operand);

                        if updated != raw {
                            machine.memory.write(address, updated);
                        }

                        ui.monospace(format!("{:04X}", raw as u16));

                        ui.end_row();
                    }
                });
        });
}

fn print_headers(ui: &mut Ui) {
    ui.label("Adr");
    ui.label("Asm (Op)");
    ui.label("Opnd");
    ui.label("Raw");
    ui.end_row();
}
