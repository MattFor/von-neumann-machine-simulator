use egui::{Align2, Button, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};

use crate::machine::{Machine, MicroOp};

const COLUMN_COUNT: usize = 12;
const TRANSFER_HEIGHT: f32 = 72.0;
const BUS_HEIGHT: f32 = 34.0;
const BUS_COLOR: Color32 = Color32::from_gray(140);
const ARROW_HEAD_HEIGHT: f32 = 16.0;

pub const HEIGHT: f32 = TRANSFER_HEIGHT + BUS_HEIGHT;

pub fn show(ui: &mut Ui, machine: &mut Machine, show_memory: bool, show_cpu: bool) -> Rect {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    let transfers = Rect::from_min_size(rect.min, vec2(rect.width(), TRANSFER_HEIGHT));

    if show_memory {
        ui.painter().add(arrow(transfer_cell(transfers, 2), true));
        ui.painter().add(arrow(transfer_cell(transfers, 3), false));

        transfer_button(ui, machine, transfers, 2, MicroOp::RamToDb);
        transfer_button(ui, machine, transfers, 3, MicroOp::DbToRam);
    }

    if show_cpu {
        transfer_button(ui, machine, transfers, 10, MicroOp::AccToDb);
        transfer_button(ui, machine, transfers, 11, MicroOp::DbToAcc);
    }

    let bus = Rect::from_min_max(pos2(rect.left(), transfers.bottom()), rect.max);
    let field = Rect::from_center_size(bus.center(), vec2(70.0, 22.0));
    ui.painter().rect_filled(bus, 0.0, BUS_COLOR);

    for (anchor, align, text) in [
        (
            field.left_center() - vec2(12.0, 0.0),
            Align2::RIGHT_CENTER,
            "Data Bus".to_string(),
        ),
        (
            field.right_center() + vec2(12.0, 0.0),
            Align2::LEFT_CENTER,
            format!("0x{:04X}", machine.cpu.mbr),
        ),
    ] {
        ui.painter().text(
            anchor,
            align,
            text,
            FontId::monospace(14.0),
            Color32::from_gray(32),
        );
    }

    let mut value = machine.cpu.mbr;

    if ui
        .put(field, egui::DragValue::new(&mut value).range(0..=0xffff))
        .on_hover_text("The value on the data bus; drag or double-click to set it by hand")
        .changed()
    {
        machine.cpu.set_mbr(value);
    }

    transfers
}

pub fn cpu_connections(grid: Rect, accumulator: Option<Rect>) -> Shape {
    let outgoing = transfer_cell(grid, 10);
    let incoming = transfer_cell(grid, 11);
    let Some(accumulator) = accumulator.filter(|rect| rect.bottom() <= grid.top()) else {
        return Shape::Vec(vec![arrow(outgoing, true), arrow(incoming, false)]);
    };

    let half_width = arrow_half_width(outgoing).min(accumulator.width() * 0.2);
    let stroke = Stroke::new(half_width * 0.9, BUS_COLOR);
    let source = pos2(
        accumulator.left() + accumulator.width() * 0.25,
        accumulator.bottom(),
    );
    let target = pos2(
        accumulator.left() + accumulator.width() * 0.75,
        accumulator.bottom(),
    );
    let incoming_bend = target.y + ARROW_HEAD_HEIGHT + 4.0;
    let outgoing_bend = incoming_bend + 10.0;
    let outgoing_tip = outgoing.center_bottom();

    Shape::Vec(vec![
        Shape::line(
            vec![
                source,
                pos2(source.x, outgoing_bend),
                pos2(outgoing.center().x, outgoing_bend),
                pos2(outgoing.center().x, outgoing_tip.y - ARROW_HEAD_HEIGHT),
            ],
            stroke,
        ),
        arrow_head(outgoing_tip, half_width, true),
        Shape::line(
            vec![
                incoming.center_bottom(),
                pos2(incoming.center().x, incoming_bend),
                pos2(target.x, incoming_bend),
                pos2(target.x, target.y + ARROW_HEAD_HEIGHT),
            ],
            stroke,
        ),
        arrow_head(target, half_width, false),
    ])
}

fn transfer_cell(grid: Rect, column: usize) -> Rect {
    let width = grid.width() / COLUMN_COUNT as f32;

    Rect::from_min_size(
        grid.min + vec2((column - 1) as f32 * width, 0.0),
        vec2(width, grid.height()),
    )
}

fn arrow_half_width(cell: Rect) -> f32 {
    (cell.width() * 0.35).min(26.0)
}

fn arrow(cell: Rect, down: bool) -> Shape {
    let center = cell.center().x;
    let half_width = arrow_half_width(cell);
    let (start, tip, direction) = if down {
        (cell.top(), cell.bottom(), 1.0)
    } else {
        (cell.bottom(), cell.top(), -1.0)
    };
    let shoulder = tip - direction * ARROW_HEAD_HEIGHT;

    Shape::Vec(vec![
        Shape::rect_filled(
            Rect::from_two_pos(
                pos2(center - half_width * 0.45, start),
                pos2(center + half_width * 0.45, shoulder),
            ),
            0.0,
            BUS_COLOR,
        ),
        arrow_head(pos2(center, tip), half_width, down),
    ])
}

fn arrow_head(tip: Pos2, half_width: f32, down: bool) -> Shape {
    let shoulder = tip.y
        + if down {
            -ARROW_HEAD_HEIGHT
        } else {
            ARROW_HEAD_HEIGHT
        };

    Shape::convex_polygon(
        vec![
            pos2(tip.x - half_width, shoulder),
            pos2(tip.x + half_width, shoulder),
            tip,
        ],
        BUS_COLOR,
        Stroke::NONE,
    )
}

fn transfer_button(ui: &mut Ui, machine: &mut Machine, grid: Rect, column: usize, op: MicroOp) {
    let cell = transfer_cell(grid, column);
    let (label, hint) = op.info();

    if ui
        .put(
            Rect::from_center_size(cell.center(), vec2((cell.width() - 4.0).min(80.0), 24.0)),
            Button::new(label),
        )
        .on_hover_text(hint)
        .clicked()
    {
        machine.manual(op);
    }
}
