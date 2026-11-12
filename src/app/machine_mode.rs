use super::{controls, gui_state::AppState};
use crate::gui::{control_unit_view, cpu_view, data_bus_view, memory_view, register_view};
use egui::Ui;

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

    egui::Panel::bottom("data_bus_panel")
        .exact_size(data_bus_view::HEIGHT)
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show(ui, |ui| {
            data_bus_view::show(
                ui,
                &mut state.machine,
                memory,
                registers || control_unit || cpu,
                advanced,
            );
        });

    // The memory table needs the most room, the registers the least.
    let weights = [
        (memory, 1.4),
        (registers, 0.8),
        (control_unit, 1.0),
        (cpu, 1.0),
    ];
    let shown = weights.iter().filter(|(shown, _)| *shown);
    let gap = ui.spacing().item_spacing.x;
    let gaps = gap * (shown.clone().count() as f32 - 1.0);
    let total: f32 = shown.map(|(_, weight)| weight).sum();

    let area = ui.available_rect_before_wrap();
    let mut left = area.left();
    let mut column = |index: usize| {
        let width = (area.width() - gaps) * weights[index].1 / total;
        let rect = egui::Rect::from_min_size(
            egui::pos2(left, area.top()),
            egui::vec2(width, area.height()),
        );

        left += width + gap;
        rect
    };

    if memory {
        let rect = column(0);

        panel(ui, rect, |ui| {
            memory_view::show(ui, &mut state.machine, advanced);
        });
    }

    if registers {
        let rect = column(1);

        scrolling(ui, rect, "registers_scroll", |ui| {
            register_view::show(ui, &state.machine);
        });
    }

    if control_unit {
        let rect = column(2);

        scrolling(ui, rect, "control_unit_scroll", |ui| {
            if let Some(message) = control_unit_view::show(ui, &mut state.machine) {
                state.status_message = message;
            }
        });
    }

    if cpu {
        let rect = column(3);

        scrolling(ui, rect, "cpu_scroll", |ui| {
            cpu_view::show(ui, &mut state.machine);
        });
    }
}
