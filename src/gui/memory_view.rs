use egui::Ui;

use crate::machine::Machine;
use crate::machine::{Instruction, MEMORY_SIZE, Opcode, encode, execute};
use crate::utilities::parse_value;

const VISIBLE_CELLS: usize = 256;
const HIGHLIGHT_DARK: egui::Color32 = egui::Color32::from_rgb(55, 30, 0);
const HIGHLIGHT_LIGHT: egui::Color32 = egui::Color32::from_rgb(255, 225, 170);
const DEFAULT_ADDRESS: &str = "0";

pub struct Header {
    pub label: &'static str,
    pub description: &'static str,
}

const COLUMN_HEADERS: [Header; 4] = [
    Header {
        label: "Adr",
        description: "Address (click one to toggle a breakpoint)",
    },
    Header {
        label: "Asm",
        description: "Opcode",
    },
    Header {
        label: "Opnd",
        description: "Operand (value)",
    },
    Header {
        label: "Val",
        description: "Whole word, decimal (hover a cell for hex)",
    },
];

pub fn show(ui: &mut Ui, machine: &mut Machine) {
    ui.heading("Memory");

    address_bus_panel(ui, machine);

    let separator_height = ui.separator().rect.height();
    let footer_height =
        ui.spacing().interact_size.y + 2.0 * ui.spacing().item_spacing.y + separator_height;

    memory_grid_panel(ui, machine, footer_height);

    let pc = machine.cpu.pc;
    let raw = machine.memory.read(pc as usize);

    ui.add_sized(
        [ui.available_width(), ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(format!("{raw}")).monospace()),
    )
    .on_hover_text("Value of the cell the address bus (PC) points at");

    ui.separator();
}

fn address_bus_panel(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        let id = egui::Id::new("address_bus_edit");
        let mut address_bus: String = ui
            .data(|data| data.get_temp(id))
            .unwrap_or(DEFAULT_ADDRESS.to_string());

        let field = ui.add(egui::TextEdit::singleline(&mut address_bus).desired_width(80.0));
        ui.data_mut(|data| data.insert_temp(id, address_bus.clone()));

        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let button = ui
            .button("Set address bus")
            .on_hover_text("Point the program counter at this address (decimal or 0x hex)");

        if (button.clicked() || enter)
            && let Some(parsed_address) = parse_value(&address_bus)
                .filter(|address| (0..MEMORY_SIZE as i32).contains(address))
        {
            execute(
                machine,
                Instruction {
                    opcode: Opcode::Jump,
                    operand: parsed_address,
                },
            );
        }
    });
}

fn memory_grid_panel(ui: &mut Ui, machine: &mut Machine, footer_height: f32) {
    let pc = machine.cpu.pc as usize;
    let row_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    let definitions = machine.instruction_set.definitions.clone();
    let column_width = (ui.available_width() - 3.0 * ui.spacing().item_spacing.x) / 4.0; // 4 columns, 3 gaps

    egui::Grid::new("memory_header")
        .striped(true)
        .show(ui, |ui| {
            print_headers(ui, row_height, column_width);
        });

    let scroll_height = (ui.available_height() - footer_height).max(0.0);

    // Follow the program counter only when it moves, so manual scrolling still works.
    let pc_id = egui::Id::new("memory_last_pc");
    let pc_moved = ui.data_mut(|data| std::mem::replace(data.get_temp_mut_or(pc_id, pc), pc)) != pc;

    let highlight = if ui.visuals().dark_mode {
        HIGHLIGHT_DARK
    } else {
        HIGHLIGHT_LIGHT
    };

    egui::ScrollArea::vertical()
        .id_salt("memory_scroll")
        .max_height(scroll_height)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .show_rows(ui, row_height, VISIBLE_CELLS, |ui, rows| {
            // NOTE: only the visible slice is rendered, so grid row indices are
            // relative to `first_address`.
            let first_address = rows.start;

            if pc_moved {
                let pitch = row_height + ui.spacing().item_spacing.y;
                let top = ui.max_rect().top() + (pc as f32 - first_address as f32) * pitch;

                ui.scroll_to_rect(
                    egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), top..=top + pitch),
                    None,
                );
            }

            egui::Grid::new("memory_grid")
                .striped(true)
                .with_row_color(move |row, _style| {
                    if pc >= first_address && row == (pc - first_address) {
                        Some(highlight)
                    } else {
                        None
                    }
                })
                .show(ui, |ui| {
                    for address in rows {
                        let raw = machine.memory.read(address);

                        let mut code = (raw >> 8) & 0xff;
                        let mut operand = raw & 0xff;
                        let mut value = raw;

                        let breakpoint = machine.breakpoints.contains(&(address as u16));
                        let label = if breakpoint {
                            egui::RichText::new(format!("● {address:04X}"))
                                .color(ui.visuals().error_fg_color)
                        } else {
                            egui::RichText::new(format!("{address:04X}"))
                        };

                        let address_cell = ui
                            .add_sized(
                                [column_width, row_height],
                                egui::Label::new(label.monospace())
                                    .truncate()
                                    .selectable(false)
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text("Click to toggle a breakpoint");

                        if address_cell.clicked() && !machine.breakpoints.remove(&(address as u16))
                        {
                            machine.breakpoints.insert(address as u16);
                        }

                        let combo_response = egui::ComboBox::from_id_salt(("opcode", address))
                            .width(column_width)
                            .selected_text(machine.instruction_set.mnemonic(code))
                            .show_ui(ui, |ui| {
                                for definition in &definitions {
                                    let response = ui.selectable_value(
                                        &mut code,
                                        definition.code,
                                        format!(
                                            "{:02X} - {}",
                                            definition.code, definition.mnemonic
                                        ),
                                    );

                                    response.on_hover_text(format!(
                                        "{} (code {:02X})\n{}",
                                        definition.mnemonic,
                                        definition.code,
                                        definition.opcode.description(),
                                    ));
                                }
                            });

                        if let Some(definition) = definitions.iter().find(|d| d.code == code) {
                            combo_response
                                .response
                                .on_hover_text(definition.opcode.description());
                        }

                        ui.add_sized(
                            [column_width, row_height],
                            egui::DragValue::new(&mut operand)
                                .range(0..=255)
                                .hexadecimal(2, false, true),
                        );

                        ui.add_sized(
                            [column_width, row_height],
                            egui::DragValue::new(&mut value)
                                .range(0..=0xffff)
                                .clamp_existing_to_range(false),
                        )
                        .on_hover_text(format!("0x{:04X}", raw as u16));

                        let updated = encode(code, operand);

                        // Compared field by field so that just showing a cell never rewrites it.
                        if updated != encode(raw >> 8, raw) {
                            machine.memory.write(address, updated);
                        } else if value != raw {
                            machine.memory.write(address, value);
                        }

                        ui.end_row();
                    }
                });
        });
}

fn print_headers(ui: &mut Ui, row_height: f32, column_width: f32) {
    for header in COLUMN_HEADERS {
        let r = ui.add_sized(
            [column_width, row_height],
            egui::Label::new(egui::RichText::new(header.label).monospace()).truncate(),
        );

        r.on_hover_text(header.description);
    }
}
