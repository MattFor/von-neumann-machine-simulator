use egui::{Align2, Button, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};

use crate::machine::{Machine, MicroOp};

const COLUMN_COUNT: usize = 12;
const TRANSFER_HEIGHT: f32 = 56.0;
const BUS_HEIGHT: f32 = 32.0;
const ARROW_HEAD_HEIGHT: f32 = 12.0;

pub const HEIGHT: f32 = TRANSFER_HEIGHT + BUS_HEIGHT;

// With `by_hand` the transfers get their buttons and the bus value can be edited.
pub fn show(ui: &mut Ui, machine: &mut Machine, show_memory: bool, show_cpu: bool, by_hand: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    let transfers = Rect::from_min_size(rect.min, vec2(rect.width(), TRANSFER_HEIGHT));
    let color = ui.visuals().selection.bg_fill;
    let ink = ui.visuals().selection.stroke.color;

    // Memory hangs on the left of the bus, the CPU on the right.
    for (shown, column, op) in [
        (show_memory, 2, MicroOp::RamToDb),
        (show_memory, 3, MicroOp::DbToRam),
        (show_cpu, 10, MicroOp::AccToDb),
        (show_cpu, 11, MicroOp::DbToAcc),
    ] {
        if !shown {
            continue;
        }

        let cell = transfer_cell(transfers, column);
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

    let bus = Rect::from_min_max(pos2(rect.left(), transfers.bottom()), rect.max);
    let field = Rect::from_center_size(bus.center(), vec2(70.0, 22.0));
    let value = machine.cpu.mbr;

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
            // Hex means nothing to a Johnny program.
            if machine.instruction_set.johnny {
                String::new()
            } else {
                format!("0x{value:04X}")
            },
        ),
        (
            field.center(),
            Align2::CENTER_CENTER,
            if by_hand {
                String::new()
            } else {
                value.to_string()
            },
        ),
    ] {
        ui.painter()
            .text(anchor, align, text, FontId::monospace(14.0), ink);
    }

    if by_hand {
        let mut value = value;

        if ui
            .put(field, egui::DragValue::new(&mut value).range(0..=0xffff))
            .on_hover_text("The value on the data bus; drag or double-click to set it by hand")
            .changed()
        {
            machine.cpu.set_mbr(value);
        }
    }
}

// Counted in from the nearer edge, and never narrower than a button label.
fn transfer_cell(grid: Rect, column: usize) -> Rect {
    let pitch = (grid.width() / COLUMN_COUNT as f32).max(84.0);
    let center = if column <= COLUMN_COUNT / 2 {
        grid.left() + (column as f32 - 0.5) * pitch
    } else {
        grid.right() - (COLUMN_COUNT as f32 + 0.5 - column as f32) * pitch
    };

    Rect::from_center_size(
        pos2(center, grid.center().y),
        vec2(pitch.min(90.0), grid.height()),
    )
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
