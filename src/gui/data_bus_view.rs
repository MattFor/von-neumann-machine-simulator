use egui::{Align2, Button, Color32, FontId, Pos2, Rect, Shape, Stroke, Ui, pos2, vec2};

use super::word_drag;
use crate::machine::{Machine, MicroOp};

const TRANSFER_HEIGHT: f32 = 56.0;
const BUS_HEIGHT: f32 = 32.0;
const ARROW_HEAD_HEIGHT: f32 = 12.0;
const ARROW_PITCH: f32 = 90.0; // room for a button label

pub const HEIGHT: f32 = TRANSFER_HEIGHT + BUS_HEIGHT;

// Each transfer sits on an arrow right under the card it reaches: the memory's under `memory`, the
// accumulator's under `acc`. Both are this frame's card rectangles, None when the card is hidden.
pub fn transfers(rect: Rect, memory: Option<Rect>, acc: Option<Rect>) -> Vec<(MicroOp, Rect)> {
    let top = rect.top() + TRANSFER_HEIGHT / 2.0;
    let mut cells = Vec::new();

    for (card, ops) in [
        (memory, [MicroOp::RamToDb, MicroOp::DbToRam]),
        (acc, [MicroOp::AccToDb, MicroOp::DbToAcc]),
    ] {
        let Some(card) = card else {
            continue;
        };

        let pitch = (card.width() / 2.0).min(ARROW_PITCH);

        for (index, op) in ops.into_iter().enumerate() {
            let center = pos2(card.center().x + (index as f32 - 0.5) * pitch, top);

            cells.push((op, Rect::from_center_size(center, vec2(pitch, TRANSFER_HEIGHT))));
        }
    }

    cells
}

// With `by_hand` the transfers get their buttons and the bus value can be edited.
pub fn show(
    ui: &mut Ui,
    rect: Rect,
    machine: &mut Machine,
    memory: Option<Rect>,
    acc: Option<Rect>,
    by_hand: bool,
) {
    let color = ui.visuals().selection.bg_fill;
    let ink = ui.visuals().selection.stroke.color;

    for (op, cell) in transfers(rect, memory, acc) {
        let onto_bus = matches!(op, MicroOp::RamToDb | MicroOp::AccToDb);

        ui.painter().add(arrow(cell, onto_bus, color));

        if by_hand {
            let (label, hint) = op.info();
            let button = Rect::from_center_size(cell.center(), vec2(cell.width() - 4.0, 24.0));

            if ui
                .put(button, Button::new(label))
                .on_hover_text(hint)
                .clicked()
            {
                machine.manual(op);
            }
        }
    }

    let bus = Rect::from_min_max(pos2(rect.left(), rect.top() + TRANSFER_HEIGHT), rect.max);
    let format = machine.instruction_set.format;
    let digits = format.code_digits + format.operand_digits;
    let value = machine.cpu.mbr;
    let mut word = format.word(value);
    let field = Rect::from_center_size(
        bus.center(),
        vec2((digits as f32 * 9.0 + 16.0).max(70.0), 22.0),
    );

    ui.painter().rect_filled(bus, 6.0, color);

    for (anchor, align, text) in [
        (
            field.left_center() - vec2(12.0, 0.0),
            Align2::RIGHT_CENTER,
            "Data Bus".to_string(),
        ),
        (
            field.right_center() + vec2(12.0, 0.0),
            Align2::LEFT_CENTER,
            // The field shows the word in the radix; in decimal it is the number already.
            match format.radix {
                10 => String::new(),
                _ => format!("= {value}"),
            },
        ),
        (
            field.center(),
            Align2::CENTER_CENTER,
            if by_hand {
                String::new()
            } else {
                format.text(word, digits)
            },
        ),
    ] {
        ui.painter()
            .text(anchor, align, text, FontId::monospace(14.0), ink);
    }

    if by_hand
        && ui
            .put(field, word_drag(&mut word, format, machine.instruction_set.values()))
            .on_hover_text(format!(
                "The word on the data bus, {value} in decimal; drag or double-click to set it by hand"
            ))
            .changed()
    {
        machine.set_data_bus(format.value(word));
    }
}

fn arrow(cell: Rect, down: bool, color: Color32) -> Shape {
    let center = cell.center().x;
    let half_width = 10.0;
    let (start, tip, direction) = if down {
        (cell.top(), cell.bottom(), 1.0)
    } else {
        (cell.bottom(), cell.top(), -1.0)
    };
    let shoulder = tip - direction * ARROW_HEAD_HEIGHT;

    Shape::Vec(vec![
        Shape::rect_filled(
            Rect::from_two_pos(
                pos2(center - half_width * 0.4, start),
                pos2(center + half_width * 0.4, shoulder),
            ),
            0.0,
            color,
        ),
        arrow_head(pos2(center, tip), half_width, shoulder, color),
    ])
}

fn arrow_head(tip: Pos2, half_width: f32, shoulder: f32, color: Color32) -> Shape {
    Shape::convex_polygon(
        vec![
            pos2(tip.x - half_width, shoulder),
            pos2(tip.x + half_width, shoulder),
            tip,
        ],
        color,
        Stroke::NONE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{FETCH, InstructionSet};

    fn marie() -> Machine {
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::marie();

        machine
    }

    #[test]
    fn ffff_typed_on_the_marie_data_bus_reaches_the_accumulator_as_minus_one() {
        let mut machine = marie();
        let format = machine.instruction_set.format;

        // What the field's parser, then its change, do with the typed text.
        for text in ["FFFF", "-1"] {
            let word = format.word(format.parse(text).unwrap());
            machine.set_data_bus(format.value(word));
            machine.manual(MicroOp::DbToAcc);

            assert_eq!(machine.cpu.acc, -1, "{text}");
        }
    }

    #[test]
    fn a_fetched_marie_word_shows_as_its_hex_word() {
        let mut machine = marie();
        machine.memory.write(0, 0x8400);

        for op in FETCH {
            machine.manual(op);
        }

        let format = machine.instruction_set.format;

        assert_eq!(machine.cpu.mbr, -0x7c00);
        assert_eq!(format.text(format.word(machine.cpu.mbr), 4), "8400");
    }
}
