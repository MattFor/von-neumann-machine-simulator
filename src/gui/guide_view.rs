use egui::Ui;

use super::listing;
use crate::app::gui_state::{PRESETS, preset_of};
use crate::machine::{DataEncoding, FETCH, Machine, MicroOp, PcOverflow, UnknownCode};

// The quick start, then "The machine", which describes the loaded one, then the rest.
const SECTIONS: [(&str, &str); 9] = [
    (
        "Quick start",
        "On the Machine tab pick a program from Example, then Run (F5) to watch it or Step \
         (F10) to go one instruction at a time. What it prints shows up in the Console.\n\n\
         The app starts in a simple view: the memory, the registers and the bus between them. \
         Advanced, top right, adds the control unit, the buttons to drive the machine by hand \
         and the view switches. Machine, next to it, picks the machine to simulate.\n\n\
         Hover over any button, column or register to see what it does.",
    ),
    (
        "Running a program",
        "Run (F5) carries out instructions at the chosen speed and becomes Pause while it \
         does.\n\
         Step (F10) carries out one instruction.\n\
         Reset (Ctrl+R) puts the registers and the console back to the start. Memory is kept.\n\
         Clear erases the memory.\n\
         Open Program (Ctrl+O) and Save Program (Ctrl+S) read and write program files.\n\n\
         A running program stops by itself on HALT, on a breakpoint, and while IN waits for a \
         number.",
    ),
    (
        "Memory",
        "Change a cell through its instruction (Asm), its operand (Opnd) or the number it holds \
         (Val, negative numbers included). Asm shows the instruction the cell runs as, so a word \
         that is no instruction shows its code with a ?. The orange row is the program counter \
         and the view follows it. The blue row is the cell the address bus points at, so the \
         one that was last read or written.\n\n\
         Click an address to put a breakpoint on it. Run stops when it gets there, press Run \
         again to carry on.\n\n\
         A program starts at address 0. To start it somewhere else, switch on Advanced, set \
         the program counter in the CPU card and press Set as entry point.",
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
         Switch on Advanced to watch this in the Control Unit card. The next step is marked \
         with >. Micro Step (F11) carries out only that one, Step finishes the instruction.",
    ),
    (
        "By hand",
        "In the advanced view every micro step is also a button, so you can play control unit \
         yourself. The transfers to and from the data bus sit on the arrows above the blue \
         bar, the arithmetic is in the CPU card and the rest in the Control Unit card.\n\n\
         Set address bus in the Memory card and the number on the blue bar put a value on a \
         bus directly. Like the accumulator in the CPU card, the bar takes a whole word in the \
         machine's radix, so on MARIE FFFF is -1. Next to the accumulator it says whether it \
         is zero, which is what TST, JZ and the =0: steps look at.",
    ),
    (
        "Your own instructions",
        "In the Control Unit card type a name, press Record, press the micro step buttons in \
         the order the instruction should do them and press Stop recording. The new instruction \
         gets the next free code and can be picked in the memory like any other. Finish with \
         pc++, or the program never moves on.\n\n\
         The Instruction Set tab lists the steps of every instruction. There you can rename \
         instructions, change their codes and add aliases. Set Operation to Custom to edit the \
         steps: + adds one at the end, clicking a step removes it. Save Set writes the set to a \
         file, and a saved program carries its set with it.\n\n\
         Tick Operand to make the operand part of an instruction, like LMC's INP (901) and OUT \
         (902), which share code 9. Tick Mask as well and only the operand bits under the mask \
         have to match, like MARIE's three Skipconds. Two instructions that match the same \
         words are marked duplicate: only the first of them runs.",
    ),
    (
        "Programs as text",
        "Open Program also reads .asm files: a program as text, one instruction per line in \
         the mnemonics of the instruction set that is loaded, like LOAD 1F. Mnemonics can be \
         in any case. Numbers are in the machine's radix, hex on the Standard machine and \
         decimal on Johnny; 0x, 0b or 0d in front makes them hex, binary or decimal.\n\n\
         Comments start with ;, # or / (MARIE's). A name with : or , after it at the start of \
         a line is a label, and can stand in for a number. Like in the Little Man Computer, a \
         line may also start with a label followed by the instruction. Instructions with a \
         fixed operand, like LMC's INP and OUT, take none; MARIE's Skipcond takes its \
         condition, as in Skipcond 400. Two mnemonics that differ only in case can't be used.\n\n\
         DAT 5 puts the number 5 in a cell, and DEC 5 and HEX 5 do it in decimal or hex. On \
         MARIE, DEC -1 is stored as FFFF. An instruction of the same name wins over any of \
         these, like Johnny's DEC. ORG 40 carries on at address 40; going back over a filled \
         cell is a mistake. END stops reading, like in MARIE, and END 40 or END start also \
         says where the program starts. Without it, it starts at the first instruction or \
         number.\n\n\
         Save a program under a name ending in .asm to write it as text. Cells that hold no \
         instruction come out as DAT, and a start other than 0 as END.\n\n\
         Files ending in .mas are MARIE's own: Open Program reads them in MARIE's mnemonics, \
         whatever machine is loaded, and switches to MARIE. Saving under .mas writes a MARIE \
         program that way.",
    ),
    (
        "Machines",
        "Machine, top right, picks what to simulate. Standard has hex words: a byte of code, \
         then a byte of operand. Johnny works like the Johnny simulator: decimal words, 1000 \
         cells and the instructions TAKE, ADD, SUB, SAVE, JMP, TST, INC, DEC, NULL and HLT. \
         Bonsai is Johnny with its minimal five instructions. The Little Man Computer has 100 \
         mailboxes of three decimal digits, and MARIE 4096 cells of 16-bit hex words. CARDIAC, \
         Bell Labs' cardboard computer, has 100 cells of a sign and three decimal digits and a \
         four-digit accumulator. Its cell 00 always holds 001, the loader: run from 00, it \
         reads the next input into 01 and carries on there. JMP leaves a jump back in cell 99, \
         and SFT 21 shifts the accumulator 2 digits left, then 1 right. Each machine keeps its \
         own program, so switching back and forth loses nothing.\n\n\
         Once the instruction set or its format matches none of them, the picker says Custom. \
         Picking the machine again gives it its own set back and keeps the memory.\n\n\
         Example has a program for every machine and loads it on its machine.\n\n\
         The Format on the Instruction Set tab sets what the words look like: the radix, the \
         digits of code and operand, how many codes and cells there are, the accumulator range, \
         how negative numbers are stored, and what happens on an unknown code or after the last \
         cell. A format the registers cannot hold, or one that leaves an instruction without \
         its code, is not applied, and the tab says why.\n\n\
         Open Program reads Johnny's .ram files and Open Set its .mc microcode files. Both \
         switch to Johnny by themselves. To write a .ram file for Johnny, save the program \
         under a name ending in .ram.",
    ),
];

// "The machine", for the one that is loaded.
fn machine_text(machine: &Machine) -> String {
    let set = &machine.instruction_set;
    let format = &set.format;
    let (max_code, max_operand, _) = set.limits();
    let last = format.operand_text(format.cells() as i32 - 1);
    let radix = match format.radix {
        2 => "binary".to_string(),
        8 => "octal".to_string(),
        10 => "decimal".to_string(),
        16 => "hex".to_string(),
        radix => format!("base {radix}"),
    };
    let (code, operand) = (format.code_text(1), format.operand_text(max_operand.min(10)));

    let negative = match format.data_encoding {
        DataEncoding::Unsigned if format.acc_min < 0 => {
            " Memory holds no negative numbers: a negative accumulator is stored as 0.".to_string()
        }
        DataEncoding::Unsigned => String::new(),
        DataEncoding::TwosComplement => format!(
            " Negative numbers are stored in two's complement, so -1 is {}.",
            format.text(format.word(-1), format.code_digits + format.operand_digits)
        ),
        DataEncoding::Lmc => {
            " A cell keeps its sign, from -999 to 999; a wider value keeps its low three digits."
                .to_string()
        }
    };
    let after_last = match format.pc_overflow {
        PcOverflow::Wrap => "it goes back to 0".to_string(),
        PcOverflow::Stay => format!("it stays on {last}"),
        PcOverflow::Halt => "the machine halts".to_string(),
    };
    let fixed = match format.fixed_cell {
        Some((cell, word)) => format!(
            " Cell {} always holds {} and can't be changed.",
            format.operand_text(cell as i32),
            format.text(word, format.code_digits + format.operand_digits)
        ),
        None => String::new(),
    };
    let unknown = match format.unknown_code {
        UnknownCode::Nop => "is skipped",
        UnknownCode::Halt => "halts the machine",
    };
    let stays = if format.stop_when_pc_stays {
        " Run stops on an instruction that leaves it where it was, like a jump to itself."
    } else {
        ""
    };
    let digits = |count: u32| match count {
        1 => "1 digit".to_string(),
        count => format!("{count} digits"),
    };

    format!(
        "The loaded machine is {}. Memory holds the program and its data, in {} cells from 0 to \
         {last}. Every cell is one word: an instruction code and an operand, usually the address \
         the instruction works on. Words are written in {radix}: {} of code, then {} of operand, \
         so {code}{operand} is code {code} with operand {operand}. There are {} codes; a word \
         whose code is no instruction {unknown}.{fixed}\n\n\
         ACC is the accumulator, where all arithmetic happens. It stays between {} and {}.\
         {negative}\n\
         PC is the program counter, the address of the next instruction. After the last cell \
         {after_last}.{stays}\n\
         IR is the instruction register, the instruction being carried out.\n\
         MAR is the address bus, it picks a memory cell.\n\
         MBR is the data bus, it carries a value between the memory and the CPU. It is the \
         blue bar under the cards.",
        preset_of(set).map_or("Custom", |preset| PRESETS[preset].0),
        format.cells(),
        digits(format.code_digits),
        digits(format.operand_digits),
        max_code + 1,
        format.acc_min,
        format.acc_max,
    )
}

const FROM_JOHNNY: [(&str, &str); 12] = [
    (
        "New / Save / Load RAM",
        "Clear, Save Program, Open Program (reads and writes .ram)",
    ),
    (
        "Macro step, Run, Pause, Reset",
        "Step, Run, Pause, Reset and the Speed slider",
    ),
    ("Show control unit", "Advanced, top right"),
    ("One micro step", "Micro Step in the Control Unit card"),
    (
        "Micro step buttons",
        "Same names, on the bus arrows and in the CPU and Control Unit cards",
    ),
    ("ManuellAb", "Set address bus in the Memory card"),
    ("ManuellDb", "The number on the data bus bar"),
    ("RAM write panel", "Edit the cells in the Memory card"),
    (
        "Record macrocode",
        "Record in the Control Unit card, or Custom on the Instruction Set tab",
    ),
    (
        "New / Save / Load microcode",
        "Johnny, Save Set, Open Set (reads .mc) on the Instruction Set tab",
    ),
    ("Bonsai mode", "Bonsai in the Machine picker"),
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

            for (index, (title, text)) in SECTIONS.into_iter().enumerate() {
                ui.heading(title);
                ui.label(text);
                ui.add_space(12.0);

                if index == 0 {
                    ui.heading("The machine");
                    ui.label(machine_text(machine));
                    ui.add_space(12.0);
                }
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
                    &format!(
                        "{} {}",
                        machine.instruction_set.format.code_text(definition.code),
                        definition.mnemonic
                    ),
                    9,
                    definition.opcode.description(),
                );
                ui.weak(format!("          {}", listing(definition.steps())));
            }

            ui.add_space(12.0);
            ui.heading("Coming from Johnny");
            ui.label(
                "Everything Johnny does is here, some of it under another name. Breakpoints, \
                 the console with IN and OUT, MUL, DIV and JZ are extra. Two things are \
                 missing: a set cannot be saved as a .mc file, and a changed fetch in a .mc \
                 file is ignored.",
            );

            for (johnny, here) in FROM_JOHNNY {
                row(ui, johnny, 30, here);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::InstructionSet;

    #[test]
    fn the_machine_section_describes_the_loaded_machine() {
        let mut machine = Machine::new();

        for (preset, facts) in [
            (InstructionSet::default(), ["is Standard", "256 cells from 0 to FF", "goes back to 0"]),
            (InstructionSet::johnny(), ["in decimal", "2 digits of code", "stays on 999"]),
            (InstructionSet::lmc(), ["1 digit of code", "-999 and 999", "keeps its sign"]),
            (InstructionSet::marie(), ["4096 cells", "100A is code 1", "-1 is FFFF"]),
            (InstructionSet::cardiac(), ["is CARDIAC", "-9999 and 9999", "Cell 00 always holds 001"]),
        ] {
            machine.instruction_set = preset;
            let text = machine_text(&machine);

            for fact in facts {
                assert!(text.contains(fact), "{fact}: {text}");
            }
        }
    }
}
