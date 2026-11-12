use egui::{Key, KeyboardShortcut, Modifiers, Ui};

use super::{action, listing, micro_buttons};
use crate::machine::{Definition, FETCH, Machine, MicroOp, Opcode};

const MICRO_STEP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F11);

// Returns a message for the status bar when a recording ends.
pub fn show(ui: &mut Ui, machine: &mut Machine) -> Option<String> {
    ui.heading("Control Unit");

    ui.label(format!(
        "Instruction: {}",
        machine.describe(machine.cpu.ir as i32)
    ))
    .on_hover_text(
        "The instruction register: its code picks the micro steps, its operand is what they work on",
    );

    if action(
        ui,
        !machine.halted,
        "Micro Step",
        "Carry out only the marked micro step",
        MICRO_STEP,
    ) {
        machine.micro_step();
    }

    // The buttons come before the list, so they stay put while it changes length.
    micro_buttons(
        ui,
        machine,
        &[
            MicroOp::PcToAb,
            MicroOp::DbToIns,
            MicroOp::InsToMc,
            MicroOp::InsToAb,
            MicroOp::InsToDb,
            MicroOp::InsToPc,
            MicroOp::InsToPcIfZero,
            MicroOp::PcInc,
            MicroOp::PcIncIfZero,
            MicroOp::McZero,
            MicroOp::Stop,
        ],
    );

    let message = record(ui, machine);

    ui.separator();

    // While fetching, the instruction about to be fetched is previewed.
    let raw = if machine.executing {
        machine.cpu.ir as i32
    } else {
        machine.memory.read(machine.cpu.pc as usize)
    };
    let code = (raw >> 8) & 0xff;

    for (title, steps, active) in [
        ("Fetch".to_string(), &FETCH[..], !machine.executing),
        (
            machine.instruction_set.mnemonic(code),
            machine.instruction_set.steps(code),
            machine.executing,
        ),
    ] {
        ui.strong(title);

        for (index, op) in steps.iter().enumerate() {
            let (label, hint) = op.info();
            let next = active && index == machine.mc;
            // A dot rather than a space, because leading spaces are trimmed.
            let marker = if next { ">" } else { "\u{b7}" };
            let text = egui::RichText::new(format!("{marker} {label}")).monospace();

            ui.label(if next {
                text.strong().color(ui.visuals().warn_fg_color)
            } else {
                text
            })
            .on_hover_text(hint);
        }
    }

    message
}

fn record(ui: &mut Ui, machine: &mut Machine) -> Option<String> {
    let mut message = None;

    ui.horizontal(|ui| {
        let id = egui::Id::new("record_name");
        let mut name: String = ui
            .data(|data| data.get_temp(id))
            .unwrap_or_else(|| "NEW".to_string());

        ui.add(egui::TextEdit::singleline(&mut name).desired_width(60.0))
            .on_hover_text("Name of the instruction to record");

        let recording = machine.recording.is_some();
        let label = if recording {
            "Stop recording"
        } else {
            "Record"
        };

        if ui
            .selectable_label(recording, label)
            .on_hover_text(
                "Record a new instruction: every micro step button pressed is noted down, \
                 and stopping adds them to the instruction set. End with pc++",
            )
            .clicked()
        {
            match machine.recording.take() {
                None => machine.recording = Some(Vec::new()),

                Some(micro) if micro.is_empty() => {
                    message = Some("Nothing recorded".to_string());
                }

                Some(micro) => {
                    let code = machine.instruction_set.free_code();

                    message = Some(format!("Recorded {name} as instruction {code:02X}"));
                    machine.instruction_set.definitions.push(Definition {
                        code,
                        mnemonic: name.clone(),
                        opcode: Opcode::Custom,
                        micro,
                    });
                }
            }
        }

        ui.data_mut(|data| data.insert_temp(id, name));
    });

    if let Some(steps) = &machine.recording {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            format!("Recording: {}", listing(steps)),
        );
    }

    message
}
