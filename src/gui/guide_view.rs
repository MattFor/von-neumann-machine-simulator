use egui::Ui;

use super::listing;
use crate::machine::{FETCH, Machine, MicroOp};

const SECTIONS: [(&str, &str); 8] = [
    (
        "Quick start",
        "On the Machine tab click Example to load a small countdown program, then Run (F5) to \
         watch it or Step (F10) to go one instruction at a time. What it prints shows up in the \
         Console.\n\n\
         Hover over any button, column or register to see what it does.",
    ),
    (
        "The machine",
        "Memory holds the program and its data. Every cell is one 16-bit word: the high byte is \
         the instruction code, the low byte is the operand, usually the address the instruction \
         works on.\n\n\
         ACC is the accumulator, where all arithmetic happens. It stays between 0 and 19999.\n\
         PC is the program counter, the address of the next instruction.\n\
         IR is the instruction register, the instruction being carried out.\n\
         MAR is the address bus, it picks a memory cell.\n\
         MBR is the data bus, it carries a value between the memory and the CPU. It is the grey \
         bar at the bottom.",
    ),
    (
        "Running a program",
        "Run (F5) carries out instructions at the chosen speed, Pause (F5) stops it.\n\
         Step (F10) carries out one instruction.\n\
         Reset (Ctrl+R) puts the registers and the console back to the start. Memory is kept.\n\
         Clear erases the memory.\n\
         Open Program (Ctrl+O) and Save Program (Ctrl+S) read and write program files.\n\n\
         A running program stops by itself on HALT, on a breakpoint, and while IN waits for a \
         number.",
    ),
    (
        "Memory",
        "Change a cell through its instruction (Asm), its operand (Opnd) or its whole value \
         (Val). The orange row is the program counter and the view follows it. The blue row is \
         the cell the address bus points at, so the one that was last read or written.\n\n\
         Click an address to put a breakpoint on it. Run stops when it gets there, press Run \
         again to carry on.\n\n\
         A program starts at address 0. To start it somewhere else, set the program counter in \
         the CPU view and press Set entry point to PC.",
    ),
    (
        "Console",
        "OUT prints the accumulator. When a program stops on IN, type one or more numbers and \
         press Enter. A running program then carries on by itself.",
    ),
    (
        "Micro steps",
        "An instruction is not one action. The control unit carries it out as a list of micro \
         steps, each of which moves one value. First come the fetch steps, the same every time: \
         they read the cell the program counter points at into the instruction register. \
         ins->mc then hands over to the steps of that instruction, and mc:=0 ends it. Most \
         instructions do pc++ just before that, the jumps set the program counter themselves.\n\n\
         Tick Control unit under Views to watch this. The next step is marked with >. \
         Micro Step (F11) carries out only that one, Step finishes the instruction.",
    ),
    (
        "By hand",
        "Every micro step is also a button, so you can play control unit yourself. The \
         transfers to and from the data bus sit above the grey bar, the arithmetic is in the \
         CPU view and the rest in the Control unit view.\n\n\
         Set address bus in the Memory view and the number on the grey bar put a value on a \
         bus directly. Next to the accumulator it says whether it is zero, which is what TST, \
         JZ and the =0: steps look at.",
    ),
    (
        "Your own instructions",
        "In the Control unit view type a name, press Record, press the micro step buttons in \
         the order the instruction should do them and press Stop recording. The new instruction \
         gets the next free code and can be picked in the memory view like any other. Finish \
         with pc++, or the program never moves on.\n\n\
         The Instruction Set tab lists the steps of every instruction. There you can rename \
         instructions, change their codes and add aliases. Set Operation to Custom to edit the \
         steps: + adds one at the end, clicking a step removes it. Save Set writes the set to a \
         file, and a saved program carries its set with it.\n\n\
         Johnny switches to the names and codes of the Johnny simulator, Bonsai to its minimal \
         five instructions.",
    ),
];

const FROM_JOHNNY: [(&str, &str); 12] = [
    ("New / Save / Load RAM", "Clear, Save Program, Open Program"),
    (
        "Macro step, Run, Pause, Reset",
        "Step, Run, Pause, Reset and the Speed slider",
    ),
    ("Show control unit", "Views: Control unit"),
    ("One micro step", "Micro Step in the Control unit view"),
    (
        "Micro step buttons",
        "Same names, on the data bus bar and in the CPU and Control unit views",
    ),
    ("ManuellAb", "Set address bus in the Memory view"),
    ("ManuellDb", "The number on the data bus bar"),
    ("RAM write panel", "Edit the cells in the Memory view"),
    (
        "Record macrocode",
        "Record in the Control unit view, or Custom on the Instruction Set tab",
    ),
    (
        "New / Save / Load microcode",
        "Restore Default, Save Set, Open Set on the Instruction Set tab",
    ),
    ("Bonsai mode", "Bonsai on the Instruction Set tab"),
    ("=0? lamp", "\"= 0\" next to the accumulator"),
];

fn row(ui: &mut Ui, key: &str, width: usize, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.monospace(format!("{key:<width$}"));
        ui.label(text);
    });
}

pub fn show(ui: &mut Ui, machine: &Machine) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_max_width(760.0);

            for (title, text) in SECTIONS {
                ui.heading(title);
                ui.label(text);
                ui.add_space(12.0);
            }

            ui.heading("Every micro step");

            for op in MicroOp::ALL {
                let (label, text) = op.info();

                row(ui, label, 11, text);
            }

            ui.add_space(12.0);
            ui.heading(format!(
                "Instructions of the {} set",
                machine.instruction_set.name
            ));
            ui.label(format!(
                "Every instruction starts with the fetch: {}",
                listing(&FETCH)
            ));

            for definition in &machine.instruction_set.definitions {
                row(
                    ui,
                    &format!("{:02X} {}", definition.code, definition.mnemonic),
                    9,
                    definition.opcode.description(),
                );
                ui.weak(format!("          {}", listing(definition.steps())));
            }

            ui.add_space(12.0);
            ui.heading("Coming from Johnny");
            ui.label(
                "Everything Johnny does is here, some of it under another name. Breakpoints, \
                 the console with IN and OUT, MUL, DIV and JZ are extra. Johnny's own .ram and \
                 .mc files cannot be opened.",
            );

            for (johnny, here) in FROM_JOHNNY {
                row(ui, johnny, 30, here);
            }
        });
}
