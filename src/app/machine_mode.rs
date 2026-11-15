use super::{controls, gui_state::AppState};
use crate::gui::{control_unit_view, cpu_view, data_bus_view, memory_view, register_view};
use egui::{Rect, Ui, pos2};

// A card filling the rectangle it is given; debug builds outline it in red.
fn panel(ui: &mut Ui, rect: egui::Rect, add: impl FnOnce(&mut Ui)) {
    let stroke = if cfg!(feature = "debug-mode") {
        egui::Stroke::new(1.0, egui::Color32::RED)
    } else {
        ui.visuals().window_stroke
    };

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        // Contents that do not fit are cut off instead of running into the next card.
        ui.shrink_clip_rect(rect);

        egui::Frame::new()
            .fill(ui.visuals().window_fill)
            .stroke(stroke)
            .corner_radius(8)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                add(ui);
            });
    });
}

fn scrolling(ui: &mut Ui, rect: egui::Rect, id: &str, add: impl FnOnce(&mut Ui)) {
    panel(ui, rect, |ui| {
        egui::ScrollArea::vertical().id_salt(id).show(ui, add);
    });
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    controls::show(ui, state);

    let advanced = state.advanced;

    if advanced {
        ui.horizontal(|ui| {
            ui.label("Views:");

            for (shown, label, hint) in [
                (
                    &mut state.show_memory,
                    "Memory",
                    "The memory cells: the program and its data",
                ),
                (
                    &mut state.show_registers,
                    "Registers",
                    "The registers and the instruction that comes next",
                ),
                (
                    &mut state.show_control_unit,
                    "Control unit",
                    "The micro steps inside an instruction: watch them, fire them by hand or record your own instruction",
                ),
                (
                    &mut state.show_cpu,
                    "CPU",
                    "Program counter, accumulator and the arithmetic buttons",
                ),
                (
                    &mut state.show_console,
                    "Console",
                    "What the program prints, and the numbers it reads",
                ),
            ] {
                ui.checkbox(shown, label).on_hover_text(hint);
            }
        });
    }

    // The simple view is always the memory next to the registers.
    let [memory, registers, control_unit, cpu] = if advanced {
        [
            state.show_memory,
            state.show_registers,
            state.show_control_unit,
            state.show_cpu,
        ]
    } else {
        [true, true, false, false]
    };

    ui.add_space(4.0);

    let (cards, bus) = layout(
        ui.available_rect_before_wrap(),
        [memory, registers, control_unit, cpu],
        ui.spacing().item_spacing.x,
    );
    let [memory, registers, control_unit, cpu] = cards;

    // The accumulator's transfers go to the CPU card, or to the registers that show it.
    data_bus_view::show(
        ui,
        bus,
        &mut state.machine,
        memory,
        cpu.or(registers),
        advanced,
    );

    if let Some(rect) = memory {
        panel(ui, rect, |ui| {
            memory_view::show(ui, &mut state.machine, advanced);
        });
    }

    if let Some(rect) = registers {
        scrolling(ui, rect, "registers_scroll", |ui| {
            register_view::show(ui, &state.machine);
        });
    }

    if let Some(rect) = control_unit {
        scrolling(ui, rect, "control_unit_scroll", |ui| {
            if let Some(message) = control_unit_view::show(ui, &mut state.machine) {
                state.status_message = message;
            }
        });
    }

    if let Some(rect) = cpu {
        scrolling(ui, rect, "cpu_scroll", |ui| {
            cpu_view::show(ui, &mut state.machine);
        });
    }
}

// The data bus along the bottom of `area` and the shown cards side by side above it, the memory
// widest: memory, registers, control unit, CPU. The arrows are placed from these same rectangles,
// so they cannot point at a card from another frame or at one that is hidden.
pub fn layout(area: Rect, shown: [bool; 4], gap: f32) -> ([Option<Rect>; 4], Rect) {
    let weights = [1.4, 0.8, 1.0, 1.0];
    let bus_top = (area.bottom() - data_bus_view::HEIGHT).max(area.top());
    let count = shown.iter().filter(|shown| **shown).count();
    let total: f32 = (0..4)
        .filter(|index| shown[*index])
        .map(|index| weights[index])
        .sum();
    let width = (area.width() - gap * count.saturating_sub(1) as f32).max(0.0);

    let mut left = area.left();
    let cards = std::array::from_fn(|index| {
        shown[index].then(|| {
            let right = left + width * weights[index] / total;
            let card = Rect::from_min_max(pos2(left, area.top()), pos2(right, bus_top));

            left = right + gap;
            card
        })
    });

    (cards, Rect::from_min_max(pos2(area.left(), bus_top), area.max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::gui_state::PRESETS;
    use crate::machine::MicroOp;
    use egui::vec2;

    #[test]
    fn every_arrow_meets_a_shown_card_and_no_card_covers_another_or_the_bus() {
        for size in [vec2(300.0, 150.0), vec2(830.0, 260.0), vec2(1160.0, 500.0), vec2(2500.0, 1400.0)] {
            let area = Rect::from_min_size(pos2(8.0, 90.0), size);

            for mask in 0..16 {
                let shown: [bool; 4] = std::array::from_fn(|index| mask & (1 << index) != 0);
                let (cards, bus) = layout(area, shown, 8.0);
                let [memory, registers, _, cpu] = cards;
                let acc = cpu.or(registers);

                assert!(area.contains_rect(bus) && (bus.height() - data_bus_view::HEIGHT).abs() < 0.01);

                for (index, card) in cards.iter().enumerate() {
                    assert_eq!(card.is_some(), shown[index], "{mask} {size:?}");

                    let Some(card) = card else {
                        continue;
                    };

                    assert!(area.expand(0.01).contains_rect(*card) && card.bottom() <= bus.top());

                    for other in cards[index + 1..].iter().flatten() {
                        assert!(!card.intersects(*other), "{mask} {size:?}");
                    }
                }

                let arrows = data_bus_view::transfers(bus, memory, acc);

                assert_eq!(arrows.len(), 2 * (memory.is_some() as usize + acc.is_some() as usize));

                for (op, arrow) in arrows {
                    let card = match op {
                        MicroOp::RamToDb | MicroOp::DbToRam => memory,
                        _ => acc,
                    }
                    .unwrap();

                    assert!(arrow.left() >= card.left() - 0.01 && arrow.right() <= card.right() + 0.01);
                    assert!(arrow.top() >= bus.top() && arrow.bottom() <= bus.bottom());
                }
            }
        }
    }

    // Draws the tabs without a window for every view of every machine, so none of them panics.
    #[test]
    fn every_view_of_every_machine_draws() {
        let ctx = egui::Context::default();
        let mut state = AppState::new();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1180.0, 760.0))),
            ..Default::default()
        };
        let draw = |state: &mut AppState, tab: fn(&mut Ui, &mut AppState)| {
            ctx.run_ui(input.clone(), |ui| tab(ui, state))
                .textures_delta
                .clear();
        };

        // Every preset, then a binary machine with 12-digit words.
        for preset in 0..=PRESETS.len() {
            if preset < PRESETS.len() {
                state.pick(preset);
            } else {
                state.pick(0);

                let set = &mut state.machine.instruction_set;
                set.definitions.retain(|definition| definition.code < 16);
                set.format.radix = 2;
                set.format.code_digits = 4;
                set.format.operand_digits = 8;
                set.check().unwrap();
            }

            state.machine.memory.write(0, -1);

            for mask in 0..32 {
                state.advanced = mask & 1 != 0;
                state.show_memory = mask & 2 != 0;
                state.show_registers = mask & 4 != 0;
                state.show_control_unit = mask & 8 != 0;
                state.show_cpu = mask & 16 != 0;

                draw(&mut state, show);
            }

            draw(&mut state, super::super::instruction_mode::show);
            draw(&mut state, |ui, state| crate::gui::guide_view::show(ui, &state.machine));
        }
    }
}
