use std::collections::HashMap;

use crate::machine::{DataEncoding, Definition, InstructionSet};

// What a mnemonic names.
enum Found<'a> {
    One(&'a Definition),
    // Instructions with fixed operands that share a first word, picked by the operand, like
    // MARIE's Skipcond 400.
    Family(Vec<&'a Definition>),
    // Mnemonics that differ only in case.
    Ambiguous(Vec<&'a Definition>),
}

// What the first word of a statement stands for.
enum Op<'a> {
    Instruction(Found<'a>),
    // A data word written in this radix.
    Data(u32),
    Org,
    End,
}

struct Statement<'a> {
    line: usize,
    address: usize,
    op: Op<'a>,
    mnemonic: &'a str,
    operand: Option<&'a str>,
}

// The one place definitions are looked up, by mnemonic in any case.
fn instruction<'a>(set: &'a InstructionSet, mnemonic: &str) -> Option<Found<'a>> {
    let named = |name: &str| name.eq_ignore_ascii_case(mnemonic);
    let mut exact: Vec<&Definition> = set
        .definitions
        .iter()
        .filter(|definition| named(&definition.mnemonic))
        .collect();

    if exact.len() > 1 {
        return Some(Found::Ambiguous(exact));
    }

    if let Some(definition) = exact.pop() {
        return Some(Found::One(definition));
    }

    let family: Vec<&Definition> = set
        .definitions
        .iter()
        .filter(|definition| {
            definition.operand.is_some()
                && definition
                    .mnemonic
                    .split_whitespace()
                    .next()
                    .is_some_and(named)
        })
        .collect();

    (!family.is_empty()).then_some(Found::Family(family))
}

// A mnemonic of the set wins over a directive of the same name.
fn op<'a>(set: &'a InstructionSet, word: &str) -> Option<Op<'a>> {
    if let Some(found) = instruction(set, word) {
        return Some(Op::Instruction(found));
    }

    match word.to_ascii_uppercase().as_str() {
        "DAT" => Some(Op::Data(set.format.radix)),
        "DEC" => Some(Op::Data(10)),
        "HEX" => Some(Op::Data(16)),
        "ORG" => Some(Op::Org),
        "END" => Some(Op::End),
        _ => None,
    }
}

// A number in the radix, unless 0x, 0b or 0d says otherwise.
pub fn number(token: &str, radix: u32) -> Option<i32> {
    let (radix, digits) = match token.get(..2).map(str::to_ascii_lowercase).as_deref() {
        Some("0x") => (16, &token[2..]),
        Some("0b") => (2, &token[2..]),
        Some("0d") => (10, &token[2..]),
        _ => (radix, token),
    };

    i32::from_str_radix(digits, radix).ok()
}

// A label starts with a letter or _, so that it never reads as a decimal number.
fn is_name(token: &str) -> bool {
    token.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && token.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// Text in the set's mnemonics to memory words from address 0 and the address the program starts
// at, or every mistake with its line.
pub fn assemble(text: &str, set: &InstructionSet) -> Result<(Vec<i32>, u16), Vec<String>> {
    let format = &set.format;
    let cells = format.cells();
    let last = format.text(cells as i32 - 1, 1);
    let (_, max_operand, max_word) = set.limits();

    // Negative data is stored like the machine stores it, in two's complement on MARIE.
    // ponytail: LMC data stays 0 to 999; allow negatives here once LMC memory is signed.
    let min_data = if format.data_encoding == DataEncoding::TwosComplement {
        -(max_word + 1) / 2
    } else {
        0
    };

    let mut labels = HashMap::new();
    let mut filled = HashMap::new();
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    let mut address = 0;
    let mut end = None;

    // First pass: the address of every statement and label.
    for (index, source) in text.lines().enumerate() {
        let line = index + 1;
        // MARIE's / also covers //.
        let code = [";", "#", "/"]
            .into_iter()
            .filter_map(|mark| source.find(mark))
            .min()
            .map_or(source, |end| &source[..end]);

        let mut label = None;
        let mut rest = code;

        // name: and MARIE's name, put a label in front.
        if let Some(end) = code.find([':', ','])
            && !code[..end].trim().contains(char::is_whitespace)
        {
            label = Some(code[..end].trim());
            rest = &code[end + 1..];
        }

        let mut words: Vec<&str> = rest.split_whitespace().collect();

        // LMC marks none: a first word that is no mnemonic, even a directive's name, followed by
        // a mnemonic or directive.
        if label.is_none()
            && words.len() > 1
            && instruction(set, words[0]).is_none()
            && op(set, words[1]).is_some()
        {
            label = Some(words.remove(0));
        }

        if let Some(name) = label {
            if !is_name(name) {
                errors.push((line, format!("{name} is not a label name")));
            } else if labels.insert(name, address).is_some() {
                errors.push((line, format!("duplicate label {name}")));
            }
        }

        let Some((&mnemonic, operands)) = words.split_first() else {
            continue;
        };

        if let Some(extra) = operands.get(1) {
            errors.push((line, format!("unexpected {extra}")));
        }

        match op(set, mnemonic) {
            None => errors.push((line, format!("unknown mnemonic {mnemonic}"))),

            Some(Op::Org) => match operands
                .first()
                .and_then(|token| number(token, format.radix))
                .filter(|target| (0..cells as i32).contains(target))
            {
                Some(target) => address = target as usize,
                None => errors.push((line, format!("ORG needs an address from 0 to {last}"))),
            },

            // Like MARIE, nothing after END is read.
            Some(Op::End) => {
                end = operands.first().map(|token| (line, *token));
                break;
            }

            Some(_) if address >= cells => {
                errors.push((line, format!("past the last cell, {last}")));
            }

            Some(op) => {
                if let Some(first) = filled.insert(address, line) {
                    let cell = format.text(address as i32, 1);

                    errors.push((line, format!("cell {cell} is already filled by line {first}")));
                }

                statements.push(Statement {
                    line,
                    address,
                    op,
                    mnemonic,
                    operand: operands.first().copied(),
                });
                address += 1;
            }
        }
    }

    // Labels first, like MARIE, so a label such as A still works in hex.
    let value = |token: &str, radix: u32, min: i32, max: i32| {
        let value = match labels.get(token) {
            Some(&address) => address as i32,
            None => number(token, radix).ok_or_else(|| {
                if is_name(token) {
                    format!("unknown label {token}")
                } else {
                    format!("{token} is not a number")
                }
            })?,
        };

        if (min..=max).contains(&value) {
            Ok(value)
        } else {
            let (min, max) = (format.text(min, 1), format.text(max, 1));

            Err(format!("{token} is out of range, {min} to {max}"))
        }
    };

    // A program starts at its first statement unless END says otherwise.
    let entry_point = match end {
        Some((line, token)) => value(token, format.radix, 0, cells as i32 - 1)
            .unwrap_or_else(|message| {
                errors.push((line, message));
                0
            }),
        None => statements
            .first()
            .map_or(0, |statement| statement.address as i32),
    };

    // Second pass: the words, now that every label is known.
    let length = statements.iter().map(|statement| statement.address + 1).max();
    let mut memory = vec![0; length.unwrap_or(0)];

    for statement in &statements {
        let mnemonic = statement.mnemonic;
        let operand = statement.operand.unwrap_or("0");

        let word = match &statement.op {
            Op::Data(radix) => value(operand, *radix, min_data, max_word)
                .map(|data| if data < 0 { data + max_word + 1 } else { data }),

            Op::Instruction(Found::One(definition)) => {
                match (definition.operand, statement.operand) {
                    (Some(_), Some(_)) => Err(format!("{mnemonic} takes no operand")),
                    (Some(fixed), None) => Ok(set.join(definition.code, fixed)),
                    (None, _) => value(operand, format.radix, 0, max_operand)
                        .map(|operand| set.join(definition.code, operand)),
                }
            }

            Op::Instruction(Found::Family(definitions)) => {
                value(operand, format.radix, 0, max_operand).and_then(|operand| {
                    definitions
                        .iter()
                        .find(|definition| definition.operand == Some(operand))
                        .map(|definition| set.join(definition.code, operand))
                        .ok_or_else(|| {
                            let choices: Vec<String> = definitions
                                .iter()
                                .filter_map(|definition| definition.operand)
                                .map(|fixed| format.text(fixed, 1))
                                .collect();

                            format!("{mnemonic} needs one of {}", choices.join(", "))
                        })
                })
            }

            Op::Instruction(Found::Ambiguous(definitions)) => {
                let names: Vec<&str> = definitions
                    .iter()
                    .map(|definition| definition.mnemonic.as_str())
                    .collect();

                Err(format!("{mnemonic} is ambiguous: {}", names.join(" or ")))
            }

            Op::Org | Op::End => unreachable!("takes no cell"),
        };

        match word {
            Ok(word) => memory[statement.address] = word,
            Err(message) => errors.push((statement.line, message)),
        }
    }

    if errors.is_empty() {
        return Ok((memory, entry_point as u16));
    }

    errors.sort_by_key(|(line, _)| *line);

    Err(errors
        .into_iter()
        .map(|(line, message)| format!("line {line}: {message}"))
        .collect())
}

// Memory as text that assembles back to the same words and entry point, without the trailing
// zero cells.
pub fn disassemble(memory: &[i32], entry_point: u16, set: &InstructionSet) -> String {
    let length = memory
        .iter()
        .rposition(|word| *word != 0)
        .map_or(0, |index| index + 1);

    let mut text: String = memory[..length]
        .iter()
        .map(|&word| line(word, set) + "\n")
        .collect();

    // The first line is cell 0, where the program starts unless END says otherwise.
    if entry_point != 0 {
        text += &format!("END {}\n", set.format.text(entry_point.into(), 1));
    }

    text
}

// The first instruction or data line that reads back as the word.
fn line(word: i32, set: &InstructionSet) -> String {
    let format = &set.format;
    let (code, operand) = set.split(word);
    // Not zero-padded: in hex a padded 0B or 0D would read as a prefix.
    let operand = format.text(operand, 1);
    let data = format!("DAT {}", format.text(word, 1));

    set.definitions
        .iter()
        .filter(|definition| definition.code == code)
        .flat_map(|definition| {
            let first = definition
                .mnemonic
                .split_whitespace()
                .next()
                .unwrap_or_default();

            // As named, as MARIE's Skipcond 400, and bare like LMC's INP.
            [
                format!("{} {operand}", definition.mnemonic),
                format!("{first} {operand}"),
                definition.mnemonic.clone(),
            ]
        })
        // DEC and HEX for a set with an instruction called DAT.
        .chain([data.clone(), format!("DEC {word}"), format!("HEX {word:X}")])
        .find(|line| assemble(line, set).is_ok_and(|(words, _)| words == [word]))
        .unwrap_or(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::program::examples;

    fn round_trip(memory: &[i32], entry_point: u16, set: &InstructionSet) {
        let text = disassemble(memory, entry_point, set);
        let length = memory
            .iter()
            .rposition(|word| *word != 0)
            .map_or(0, |index| index + 1);

        assert_eq!(
            assemble(&text, set),
            Ok((memory[..length].to_vec(), entry_point)),
            "{}\n{text}",
            set.name
        );
    }

    // Assembles a program, round trips it and runs it on one input.
    fn run(lines: &[&str], set: InstructionSet, input: i32) -> (Vec<i32>, u16, String) {
        let (memory, entry_point) = assemble(&lines.join("\n"), &set).unwrap();
        round_trip(&memory, entry_point, &set);

        let mut machine = Machine::new();
        machine.instruction_set = set;
        machine.memory.load(&memory);
        machine.entry_point = entry_point;
        machine.reset();
        machine.push_input(input);
        machine.run_steps(1000);

        assert!(machine.halted);
        (memory, entry_point, machine.output)
    }

    #[test]
    fn bundled_examples_round_trip_with_their_presets() {
        for example in examples() {
            let program = &example.program;

            round_trip(&program.memory, program.entry_point, &example.preset);
        }
    }

    #[test]
    fn every_kind_of_word_round_trips() {
        let mut unreadable = InstructionSet::default();
        unreadable.definitions[2].mnemonic = "load".to_string();
        unreadable.definitions[3].mnemonic = "DAT".to_string();
        unreadable.definitions[4].mnemonic = "SU B".to_string();
        unreadable.definitions[5].mnemonic = String::new();

        for set in InstructionSet::presets().into_iter().chain([unreadable]) {
            let words = set.limits().2 + 1;
            let cells = set.format.cells() as i32;
            // An odd stride reaches every operand and most codes.
            let memory: Vec<i32> = (0..cells)
                .map(|address| (address * 7919 + 1) % words)
                .collect();

            round_trip(&memory, (cells - 1) as u16, &set);
        }
    }

    #[test]
    fn words_of_no_instruction_are_data_and_trailing_zeros_are_left_out() {
        let text = disassemble(&[0x0105, 0x2000, 0x0B0D, 0, 0], 0, &InstructionSet::default());

        assert_eq!(text, "LOAD 5\nDAT 2000\nINC D\n");
    }

    #[test]
    fn instructions_with_a_fixed_operand_are_written_without_it() {
        let lmc = InstructionSet::lmc();
        let marie = InstructionSet::marie();

        assert_eq!(assemble("INP\nout\nHLT", &lmc), Ok((vec![901, 902, 0], 0)));
        assert_eq!(disassemble(&[901, 902, 0, 5], 0, &lmc), "INP\nOUT\nHLT\nDAT 5\n");

        // MARIE writes the condition as the operand.
        assert_eq!(
            assemble("Skipcond 000\nSkipcond 400\nskipcond 800", &marie),
            Ok((vec![0x8000, 0x8400, 0x8800], 0))
        );
        assert_eq!(disassemble(&[0x8400], 0, &marie), "Skipcond 400\n");
    }

    #[test]
    fn comments_and_labels_ending_in_a_colon() {
        let text = [
            "; counts down from 5",
            "start:  LOAD count   # into the accumulator",
            "        out          // print it",
            "        DEC count",
            "        JZ end",
            "        JMP start",
            "end:    HALT",
            "count:  DAT 5",
        ]
        .join("\n");

        assert_eq!(
            assemble(&text, &InstructionSet::default()),
            Ok((vec![0x0106, 0x0A00, 0x0C06, 0x0805, 0x0700, 0xFF00, 5], 0))
        );
    }

    #[test]
    fn lmc_labels_are_a_first_word_that_is_no_mnemonic() {
        let text = [
            "        LDA ONE",
            "LOOP    ADD ONE",
            "        STA COUNT",
            "        BRA LOOP",
            "ONE     DAT 1",
            "COUNT   DAT",
        ]
        .join("\n");

        assert_eq!(
            assemble(&text, &InstructionSet::lmc()),
            Ok((vec![504, 104, 305, 601, 1, 0], 0))
        );
    }

    #[test]
    fn marie_labels_end_in_a_comma_and_org_moves_on() {
        let text = [
            "        ORG 100     / the program starts here",
            "        Load X",
            "        Add Y",
            "        Store Z",
            "        Halt",
            "X,      DEC 35",
            "Y,      HEX 1F",
            "Z,      DEC 0",
        ]
        .join("\n");
        let (memory, entry_point) = assemble(&text, &InstructionSet::marie()).unwrap();

        assert_eq!(entry_point, 0x100);
        assert_eq!(memory.len(), 0x107);
        assert!(memory[..0x100].iter().all(|word| *word == 0));
        assert_eq!(memory[0x100..], [0x1104, 0x3105, 0x2106, 0x7000, 35, 0x1F, 0]);
    }

    #[test]
    fn a_textbook_lmc_program_runs() {
        let (memory, _, output) = run(
            &[
                "// counts down from the input to zero",
                "        INP",
                "LOOP    OUT",
                "        BRZ DONE    ; stop after 0",
                "        SUB ONE",
                "        BRP LOOP",
                "DONE    HLT",
                "ONE     DAT 1",
            ],
            InstructionSet::lmc(),
            3,
        );

        assert_eq!(memory, [901, 902, 705, 206, 801, 0, 1]);
        assert_eq!(output, "3\n2\n1\n0\n");
    }

    #[test]
    fn a_textbook_marie_program_runs() {
        let (memory, entry_point, output) = run(
            &[
                "/ Adds the numbers from the input down to 1 in a subroutine",
                "        ORG 100",
                "        Input",
                "        Store N",
                "Loop,   Load N",
                "        Skipcond 800    / carry on while N > 0",
                "        Jump Done",
                "        JnS AddN",
                "        Jump Loop",
                "Done,   Load Sum",
                "        Output",
                "        Halt",
                "N,      DEC 0",
                "Sum,    DEC 0",
                "One,    HEX 1",
                "AddN,   HEX 0           / JnS keeps the return address here",
                "        Load Sum",
                "        Add N",
                "        Store Sum",
                "        Load N",
                "        Subt One",
                "        Store N",
                "        JumpI AddN",
                "        END",
            ],
            InstructionSet::marie(),
            3,
        );

        assert_eq!(entry_point, 0x100);
        assert_eq!(
            memory[0x100..],
            [
                0x5000, 0x210A, 0x110A, 0x8800, 0x9107, 0x010D, 0x9102, 0x110B, 0x6000, 0x7000, 0,
                0, 1, 0, 0x110B, 0x310A, 0x210B, 0x110A, 0x410C, 0x210A, 0xC10D,
            ]
        );
        assert_eq!(output, "6\n");
    }

    #[test]
    fn numbers_are_in_the_radix_unless_a_prefix_says_otherwise() {
        let text = "a: DAT 10\nDAT 0x10\nDAT 0d10\nDAT 0b10\nDAT 0B11\nDAT\nLOAD a";

        assert_eq!(
            assemble(text, &InstructionSet::default()),
            Ok((vec![16, 16, 10, 2, 3, 0, 0x0100], 0))
        );
        assert_eq!(
            assemble("DAT 10\nDAT 0x10\nDEC 7", &InstructionSet::johnny()),
            Ok((vec![10, 16, 8007], 0))
        );
        // MARIE keeps negative data in two's complement.
        assert_eq!(
            assemble("DEC -23\nHEX FFE9", &InstructionSet::marie()),
            Ok((vec![0xFFE9, 0xFFE9], 0))
        );
    }

    #[test]
    fn mnemonics_win_over_directives_and_lmc_labels_may_be_named_like_one() {
        let mut set = InstructionSet::default();
        set.definitions[3].mnemonic = "DAT".to_string();

        assert_eq!(assemble("DAT 5\nHEX 5", &set), Ok((vec![0x0305, 5], 0)));
        assert_eq!(
            assemble("DEC     LDA DEC\n        BRA DEC", &InstructionSet::lmc()),
            Ok((vec![500, 600], 0))
        );
    }

    #[test]
    fn end_sets_where_the_program_starts() {
        let set = InstructionSet::default();
        let entry = |text: &str| assemble(text, &set).map(|(_, entry_point)| entry_point);

        assert_eq!(assemble("NOP\nEND 5\nnot read", &set), Ok((vec![0], 5)));
        assert_eq!(entry("ORG 10\nstart: NOP\nNOP\nEND start"), Ok(0x10));
        assert_eq!(entry("ORG 10\nNOP\nEND"), Ok(0x10));
        assert_eq!(disassemble(&[1], 5, &set), "NOP 1\nEND 5\n");
    }

    #[test]
    fn every_mistake_is_reported_with_its_line() {
        let mut ambiguous = InstructionSet::default();
        ambiguous.definitions[2].mnemonic = "load".to_string();

        for (text, set, error) in [
            ("NOP\nFOO 1", None, "line 2: unknown mnemonic FOO"),
            ("JMP nowhere", None, "line 1: unknown label nowhere"),
            ("a: NOP\na: NOP", None, "line 2: duplicate label a"),
            ("LOAD 100", None, "line 1: 100 is out of range, 0 to FF"),
            ("DAT 10000", None, "line 1: 10000 is out of range, 0 to FFFF"),
            ("DAT -1", None, "line 1: -1 is out of range, 0 to FFFF"),
            ("ORG 100", None, "line 1: ORG needs an address from 0 to FF"),
            ("ORG FF\nNOP\nNOP", None, "line 3: past the last cell, FF"),
            ("DAT 1\nDAT 2\nORG 1\nDAT 3", None, "line 4: cell 1 is already filled by line 2"),
            ("END 100", None, "line 1: 100 is out of range, 0 to FF"),
            ("LOAD 1 2", None, "line 1: unexpected 2"),
            ("1x: NOP", None, "line 1: 1x is not a label name"),
            ("DAT 1G", None, "line 1: 1G is not a number"),
            ("LOAD 5", Some(&ambiguous), "line 1: LOAD is ambiguous: LOAD or load"),
            ("INP 5", Some(&InstructionSet::lmc()), "line 1: INP takes no operand"),
            (
                "Skipcond 100",
                Some(&InstructionSet::marie()),
                "line 1: Skipcond needs one of 0, 400, 800",
            ),
        ] {
            let set = set.cloned().unwrap_or_default();

            assert_eq!(assemble(text, &set), Err(vec![error.to_string()]), "{text}");
        }

        assert_eq!(
            assemble("JMP nowhere\nFOO", &InstructionSet::default()),
            Err(vec![
                "line 1: unknown label nowhere".to_string(),
                "line 2: unknown mnemonic FOO".to_string(),
            ])
        );
    }
}
