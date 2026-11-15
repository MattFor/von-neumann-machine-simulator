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
    let value = format.value(machine.read(machine.address(machine.cpu.mar as i32) as usize));
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
    // The cells the fetch and ram->db use, also when a smaller format left the registers outside.
    let pc = machine.address(machine.cpu.pc as i32) as usize;
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

    // The offset, height and rows the view was shown with last frame.
    let view_id = egui::Id::unique("memory_view");
    let (offset, height, shown_cells) = ui
        .data(|data| data.get_temp::<(f32, f32, usize)>(view_id))
        .unwrap_or((0.0, 0.0, cells));

    // Follow the program counter only when it moves, so manual scrolling still works. With more
    // or fewer rows than last frame it waits a frame: the scroll area still has the old ones, and
    // pulls back an offset past their end over the next frames.
    let target = (shown_cells == cells)
        .then(|| {
            let pc_id = egui::Id::unique("memory_last_pc");
            let pc_moved = ui.data_mut(|data| std::mem::replace(data.get_temp_mut_or(pc_id, pc), pc)) != pc;

            // An address bus set by hand is shown once too.
            ui.data_mut(|data| data.remove_temp::<usize>(egui::Id::unique(SCROLL_TO)))
                .or(pc_moved.then_some(pc))
        })
        .flatten();

    let address_bus = machine.address(machine.cpu.mar as i32) as usize;
    let (highlight, address_bus_highlight) = if ui.visuals().dark_mode {
        (HIGHLIGHT_DARK, ADDRESS_BUS_DARK)
    } else {
        (HIGHLIGHT_LIGHT, ADDRESS_BUS_LIGHT)
    };

    let mut scroll = egui::ScrollArea::vertical()
        .id_salt("memory_scroll")
        .max_height(scroll_height)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false]);

    // Set before the rows are laid out: a scroll_to request lands a frame late, so another one
    // the next frame, while running or typing an address, would be measured from the old place.
    if let Some(target) = target {
        let pitch = row_height + ui.spacing().item_spacing.y;
        let top = target as f32 * pitch;

        scroll = scroll.vertical_scroll_offset(offset.max(top + pitch - height).min(top));
    }

    let output = scroll
        .show_rows(ui, row_height, cells, |ui, rows| {
            // NOTE: only the visible slice is rendered, so grid row indices are
            // relative to `first_address`.
            let first_address = rows.start;

            // So that a value like MARIE's -32768 fits its column in the smallest window.
            ui.spacing_mut().button_padding.x = 4.0;

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
                        let cell = machine.read(address);
                        let fixed = machine.is_fixed(address);
                        let locked = format!(
                            "This cell always holds {}, like CARDIAC's loader, so it can't be changed",
                            set.format.text(cell, set.format.code_digits + set.format.operand_digits)
                        );

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
                            .on_hover_text(if fixed {
                                "This cell always holds the same word and can't be changed. \
                                 Click to toggle a breakpoint"
                            } else {
                                "Click to toggle a breakpoint"
                            });

                        if address_cell.clicked() && !machine.breakpoints.remove(&(address as u16))
                        {
                            machine.breakpoints.insert(address as u16);
                        }

                        let mut picked = None;
                        // Truncated to the width left, which the button padding comes on top of.
                        let combo_response = ui
                            .scope(|ui| {
                                ui.set_max_width(column_width - 2.0 * ui.spacing().button_padding.x);

                                if fixed {
                                    ui.disable();
                                }

                                egui::ComboBox::from_id_salt(("opcode", address))
                                    .width(column_width)
                                    .truncate()
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
                                    })
                            })
                            .inner;

                        let combo_response = combo_response.response.on_disabled_hover_text(&locked);

                        // The whole name, which a narrow column cuts short.
                        if let Some(index) = decoded {
                            let definition = &set.definitions[index];

                            combo_response.on_hover_text(format!(
                                "{}\n{}",
                                definition.mnemonic,
                                definition.opcode.description()
                            ));
                        }

                        // A code past the last one has no words to give another operand.
                        let operand_field = ui
                            .add_enabled_ui(code <= max_code && !fixed, |ui| {
                                ui.add_sized(
                                    [column_width, row_height],
                                    radix_drag(&mut operand, set.format, set.format.operand_digits)
                                        .range(0..=max_operand),
                                )
                            })
                            .inner;
                        let operand_changed = if fixed {
                            operand_field.on_disabled_hover_text(&locked)
                        } else {
                            operand_field
                        }
                        .changed();

                        let value_changed = ui
                            .add_enabled_ui(!fixed, |ui| {
                                ui.add_sized(
                                    [column_width, row_height],
                                    egui::DragValue::new(&mut value)
                                        .range(set.values())
                                        .clamp_existing_to_range(false),
                                )
                            })
                            .inner
                            .on_hover_text(machine.describe(set.format.fetched(cell)))
                            .on_disabled_hover_text(&locked)
                            .changed();

                        // Only an edit writes, so that just showing a cell never rewrites it.
                        if let Some(definition) = picked {
                            machine.edit(address, pick(&set, definition, operand));
                        } else if operand_changed {
                            machine.edit(address, set.join(code, operand));
                        } else if value_changed {
                            machine.edit(address, set.format.word(value));
                        }

                        ui.end_row();
                    }
                });
        });

    let shown = (output.state.clamped_offset().y, output.inner_rect.height(), cells);

    ui.data_mut(|data| data.insert_temp(view_id, shown));
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

    // Draws the memory view a few frames in a rect `width` wide and returns what it drew.
    fn draw(
        ctx: &egui::Context,
        machine: &mut Machine,
        width: f32,
        by_hand: bool,
        frames: usize,
    ) -> (f32, Vec<egui::epaint::ClippedShape>) {
        let mut used = 0.0;
        let mut shapes = Vec::new();

        for _ in 0..frames {
            let input = egui::RawInput {
                time: Some(ctx.input(|input| input.time) + 1.0 / 60.0),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                let rect = egui::Rect::from_min_size(ui.min_rect().min, egui::vec2(width, 300.0));

                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    show(ui, machine, by_hand);
                    used = ui.min_rect().width();
                });
            });

            output.textures_delta.clear();
            shapes = output.shapes;
        }

        (used, shapes)
    }

    // A long name like MARIE's Skipcond >0 once pushed the card past its border.
    #[test]
    fn the_memory_fits_the_width_of_the_smallest_window() {
        let ctx = egui::Context::default();
        crate::gui::theme::apply(&ctx);

        for set in InstructionSet::presets() {
            let mut machine = Machine::new();
            let words: Vec<i32> = set
                .definitions
                .iter()
                .map(|definition| pick(&set, definition, 0))
                .collect();
            machine.memory.load(&words);
            machine.instruction_set = set;

            // 249 is the memory's share of the 854 wide window with every card shown.
            for width in [249.0, 300.0] {
                for by_hand in [false, true] {
                    // The grid sizes its columns from the frame before.
                    let (used, _) = draw(&ctx, &mut machine, width, by_hand, 3);

                    assert!(used <= width + 0.5, "{} {width}: {used}", machine.instruction_set.name);
                }
            }
        }
    }

    // Whether the address shows inside the view once it has settled.
    fn shows(ctx: &egui::Context, machine: &mut Machine, address: &str) -> bool {
        let (_, shapes) = draw(ctx, machine, 300.0, true, 60);

        shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Text(text) => {
                text.galley.text() == address
                    && clipped.clip_rect.contains(text.visual_bounding_rect().center())
            }
            _ => false,
        })
    }

    // Typing 750 as the program counter moves it to 7, 75 and 750 on three frames in a row.
    #[test]
    fn the_view_lands_on_a_program_counter_that_moves_every_frame() {
        let ctx = egui::Context::default();
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();

        for pc in [9, 7, 75, 750] {
            machine.cpu.pc = pc;
            draw(&ctx, &mut machine, 300.0, true, 1);
        }

        assert!(shows(&ctx, &mut machine, "750"));
    }

    // Picking Johnny again after a 16 cell format of it, with the program counter at 265.
    #[test]
    fn the_view_lands_on_the_program_counter_when_the_cells_change() {
        let ctx = egui::Context::default();
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        machine.instruction_set.format.cells = Some(16);
        machine.cpu.pc = 265;

        draw(&ctx, &mut machine, 300.0, true, 10);
        machine.instruction_set = InstructionSet::johnny();

        assert!(shows(&ctx, &mut machine, "265"));
    }
}
