use egui::Ui;

use super::radix_drag;
use crate::assembler::number;
use crate::machine::{Definition, InstructionSet, Machine};

const HIGHLIGHT_DARK: egui::Color32 = egui::Color32::from_rgb(55, 30, 0);
const HIGHLIGHT_LIGHT: egui::Color32 = egui::Color32::from_rgb(255, 225, 170);
const ADDRESS_BUS_DARK: egui::Color32 = egui::Color32::from_rgb(0, 40, 65);
const ADDRESS_BUS_LIGHT: egui::Color32 = egui::Color32::from_rgb(190, 225, 255);
const SCROLL_TO: &str = "memory_scroll_to";
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
        description: "The instruction in this cell",
    },
    Header {
        label: "Opnd",
        description: "Operand: usually the address the instruction works on",
    },
    Header {
        label: "Val",
        description: "The number the cell holds, negative ones included (hover a cell to read it as an instruction)",
    },
];

// The code and operand a fetch of the cell gives, and which definition the executor runs for them.
pub fn decode(set: &InstructionSet, cell: i32) -> (i32, i32, Option<usize>) {
    let (code, operand) = set.split(set.format.fetched(cell));
    let found = set.find_instruction(code, operand);
    let index = set
        .definitions
        .iter()
        .position(|definition| found.is_some_and(|found| std::ptr::eq(found, definition)));

    (code, operand, index)
}

// Picking an instruction keeps the operand, unless the instruction has a fixed one.
// ponytail: an instruction without one can then decode as another of its code that takes this operand.
pub fn pick(set: &InstructionSet, definition: &Definition, operand: i32) -> i32 {
    set.join(definition.code, definition.operand.unwrap_or(operand))
}

// With `by_hand` the address bus can be set and the cell it points at is shown.
pub fn show(ui: &mut Ui, machine: &mut Machine, by_hand: bool) {
    ui.heading("Memory");

    if !by_hand {
        memory_grid_panel(ui, machine, 0.0);
        return;
    }

    address_bus_panel(ui, machine);

    let separator_height = ui.separator().rect.height();
    let footer_height =
        ui.spacing().interact_size.y + 2.0 * ui.spacing().item_spacing.y + separator_height;

    memory_grid_panel(ui, machine, footer_height);

    let format = machine.instruction_set.format;
    let value = format.value(machine.memory.read(machine.cpu.mar as usize));
    let text = match format.radix {
        10 => value.to_string(),
        _ => format!(
            "{} = {value}",
            format.text(format.word(value), format.code_digits + format.operand_digits)
        ),
    };

    ui.add_sized(
        [ui.available_width(), ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(text).monospace()),
    )
    .on_hover_text("Value of the cell the address bus points at");

    ui.separator();
}

fn address_bus_panel(ui: &mut Ui, machine: &mut Machine) {
    ui.horizontal(|ui| {
        let id = egui::Id::unique("address_bus_edit");
        let mut address_bus: String = ui
            .data(|data| data.get_temp(id))
            .unwrap_or(DEFAULT_ADDRESS.to_string());

        let field = ui.add(egui::TextEdit::singleline(&mut address_bus).desired_width(80.0));
        ui.data_mut(|data| data.insert_temp(id, address_bus.clone()));

        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let button = ui
            .button("Set address bus")
            .on_hover_text(
                "Put this address (in the machine's radix, or after 0x, 0b or 0d) on the address bus; \
                 ram->db and db->ram work on that cell",
            );

        let format = machine.instruction_set.format;
        let last = format.cells() as i32 - 1;

        if (button.clicked() || enter)
            && let Some(parsed_address) =
                number(address_bus.trim(), format.radix).filter(|address| (0..=last).contains(address))
        {
            machine.cpu.mar = parsed_address as u16;
            ui.data_mut(|data| data.insert_temp(egui::Id::unique(SCROLL_TO), parsed_address as usize));
        }
    });
}

fn memory_grid_panel(ui: &mut Ui, machine: &mut Machine, footer_height: f32) {
    let pc = machine.cpu.pc as usize;
    let row_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    let set = machine.instruction_set.clone();
    let (max_code, max_operand, _) = set.limits();
    let cells = set.format.cells();
    let column_width = (ui.available_width() - 3.0 * ui.spacing().item_spacing.x) / 4.0; // 4 columns, 3 gaps

    egui::Grid::new("memory_header")
        .striped(true)
        .show(ui, |ui| {
            print_headers(ui, row_height, column_width);
        });

    let scroll_height = (ui.available_height() - footer_height).max(0.0);

    // Follow the program counter only when it moves, so manual scrolling still works.
    let pc_id = egui::Id::unique("memory_last_pc");
    let pc_moved = ui.data_mut(|data| std::mem::replace(data.get_temp_mut_or(pc_id, pc), pc)) != pc;

    // An address bus set by hand is shown once too.
    let target = ui
        .data_mut(|data| data.remove_temp::<usize>(egui::Id::unique(SCROLL_TO)))
        .or(pc_moved.then_some(pc));

    let address_bus = machine.cpu.mar as usize;
    let (highlight, address_bus_highlight) = if ui.visuals().dark_mode {
        (HIGHLIGHT_DARK, ADDRESS_BUS_DARK)
    } else {
        (HIGHLIGHT_LIGHT, ADDRESS_BUS_LIGHT)
    };

    egui::ScrollArea::vertical()
        .id_salt("memory_scroll")
        .max_height(scroll_height)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .show_rows(ui, row_height, cells, |ui, rows| {
            // NOTE: only the visible slice is rendered, so grid row indices are
            // relative to `first_address`.
            let first_address = rows.start;

            if let Some(target) = target {
                let pitch = row_height + ui.spacing().item_spacing.y;
                let top = ui.max_rect().top() + (target as f32 - first_address as f32) * pitch;

                ui.scroll_to_rect(
                    egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), top..=top + pitch),
                    None,
                );
            }

            egui::Grid::new("memory_grid")
                .striped(true)
                .with_row_color(move |row, _style| {
                    if row + first_address == pc {
                        Some(highlight)
                    } else if row + first_address == address_bus {
                        Some(address_bus_highlight)
                    } else {
                        None
                    }
                })
                .show(ui, |ui| {
                    for address in rows {
                        let cell = machine.memory.read(address);

                        let (code, mut operand, decoded) = decode(&set, cell);
                        let mut value = set.format.value(cell);
                        let shown_address = set.format.operand_text(address as i32);

                        let breakpoint = machine.breakpoints.contains(&(address as u16));
                        let label = if breakpoint {
                            egui::RichText::new(format!("● {shown_address}"))
                                .color(ui.visuals().error_fg_color)
                        } else {
                            egui::RichText::new(shown_address)
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

                        let mut picked = None;
                        let combo_response = egui::ComboBox::from_id_salt(("opcode", address))
                            .width(column_width)
                            .selected_text(set.mnemonic_for(code, operand))
                            .show_ui(ui, |ui| {
                                for (index, definition) in set.definitions.iter().enumerate() {
                                    // A fixed operand is part of the instruction, like LMC's 9.02 OUT.
                                    let shown_code = match definition.operand {
                                        Some(fixed) => format!(
                                            "{}.{}",
                                            set.format.code_text(definition.code),
                                            set.format.operand_text(fixed)
                                        ),
                                        None => set.format.code_text(definition.code),
                                    };
                                    let response = ui.selectable_label(
                                        decoded == Some(index),
                                        format!("{shown_code} - {}", definition.mnemonic),
                                    );

                                    if response.clicked() {
                                        picked = Some(definition);
                                    }

                                    response.on_hover_text(format!(
                                        "{} (code {shown_code})\n{}",
                                        definition.mnemonic,
                                        definition.opcode.description(),
                                    ));
                                }
                            });

                        if let Some(index) = decoded {
                            combo_response
                                .response
                                .on_hover_text(set.definitions[index].opcode.description());
                        }

                        // A code past the last one has no words to give another operand.
                        let operand_changed = ui
                            .add_enabled_ui(code <= max_code, |ui| {
                                ui.add_sized(
                                    [column_width, row_height],
                                    radix_drag(&mut operand, set.format, set.format.operand_digits)
                                        .range(0..=max_operand),
                                )
                            })
                            .inner
                            .changed();

                        let value_changed = ui
                            .add_sized(
                                [column_width, row_height],
                                egui::DragValue::new(&mut value)
                                    .range(set.values())
                                    .clamp_existing_to_range(false),
                            )
                            .on_hover_text(machine.describe(set.format.fetched(cell)))
                            .changed();

                        // Only an edit writes, so that just showing a cell never rewrites it.
                        if let Some(definition) = picked {
                            machine.memory.write(address, pick(&set, definition, operand));
                        } else if operand_changed {
                            machine.memory.write(address, set.join(code, operand));
                        } else if value_changed {
                            machine.memory.write(address, set.format.word(value));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mnemonic(set: &InstructionSet, cell: i32) -> Option<&str> {
        decode(set, cell).2.map(|index| set.definitions[index].mnemonic.as_str())
    }

    #[test]
    fn cells_are_decoded_by_code_and_operand_like_the_executor() {
        let (lmc, marie) = (InstructionSet::lmc(), InstructionSet::marie());

        assert_eq!(mnemonic(&lmc, 901), Some("INP"));
        assert_eq!(mnemonic(&lmc, 902), Some("OUT"));
        assert_eq!(mnemonic(&lmc, 0), Some("HLT"));
        // Runs as no instruction, so it is not HLT.
        assert_eq!(mnemonic(&lmc, 5), None);
        assert_eq!(mnemonic(&lmc, -1), None);
        assert_eq!(mnemonic(&marie, 0x8800), Some("Skipcond >0"));
        assert_eq!(mnemonic(&marie, 0x8401), Some("Skipcond =0"));
        assert_eq!(mnemonic(&marie, 0x8000), Some("Skipcond <0"));
        assert_eq!(mnemonic(&InstructionSet::default(), 0x010a), Some("LOAD"));
    }

    fn named<'a>(set: &'a InstructionSet, name: &str) -> &'a Definition {
        set.definitions
            .iter()
            .find(|definition| definition.mnemonic == name)
            .unwrap()
    }

    #[test]
    fn picking_an_instruction_writes_its_fixed_operand() {
        let (lmc, marie) = (InstructionSet::lmc(), InstructionSet::marie());

        assert_eq!(pick(&lmc, named(&lmc, "OUT"), 0), 902);
        assert_eq!(pick(&lmc, named(&lmc, "INP"), 2), 901);
        assert_eq!(pick(&lmc, named(&lmc, "ADD"), 42), 142);
        assert_eq!(pick(&marie, named(&marie, "Skipcond =0"), 0x123), 0x8400);
        assert_eq!(mnemonic(&lmc, pick(&lmc, named(&lmc, "OUT"), 0)), Some("OUT"));
    }
}
