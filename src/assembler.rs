use std::collections::HashMap;

use crate::machine::InstructionSet;

// What the first word of a statement stands for.
#[derive(Clone, Copy)]
enum Op {
    // A code, and the operand of an instruction that takes none in assembly.
    Instruction(i32, Option<i32>),
    // A data word written in this radix.
    Data(u32),
    Org,
}

struct Statement<'a> {
    line: usize,
    address: usize,
    op: Op,
    mnemonic: &'a str,
    operand: Option<&'a str>,
}

// The one place definitions are looked up, by mnemonic in any case.
// ponytail: no fixed operands yet; LMC's INP = 901 would return (9, Some(1)) here.
fn instruction(set: &InstructionSet, mnemonic: &str) -> Option<(i32, Option<i32>)> {
    set.definitions
        .iter()
        .find(|definition| definition.mnemonic.eq_ignore_ascii_case(mnemonic))
        .map(|definition| (definition.code, None))
}

// DAT and ORG are always directives, DEC and HEX only where no instruction has the name, like
// Johnny's DEC.
fn op(set: &InstructionSet, word: &str) -> Option<Op> {
    match word.to_ascii_uppercase().as_str() {
        "DAT" => Some(Op::Data(set.format.radix)),
        "ORG" => Some(Op::Org),
        name => instruction(set, word)
            .map(|(code, fixed)| Op::Instruction(code, fixed))
            .or(match name {
                "DEC" => Some(Op::Data(10)),
                "HEX" => Some(Op::Data(16)),
                _ => None,
            }),
    }
}

// A number in the radix, unless 0x, 0b or 0d says otherwise.
fn number(token: &str, radix: u32) -> Option<i32> {
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

// Text in the set's mnemonics to memory words from address 0, or every mistake with its line.
pub fn assemble(text: &str, set: &InstructionSet) -> Result<Vec<i32>, Vec<String>> {
    let format = &set.format;
    let cells = format.cells();
    let last = format.text(cells as i32 - 1, 1);
    let (_, max_operand, max_word) = set.limits();

    let mut labels = HashMap::new();
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    let mut address = 0;

    // First pass: the address of every statement and label.
    for (index, source) in text.lines().enumerate() {
        let line = index + 1;
        let code = [";", "#", "//"]
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

        // LMC marks none: a first word that is not a mnemonic, followed by one.
        if label.is_none()
            && words.len() > 1
            && op(set, words[0]).is_none()
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

            Some(_) if address >= cells => {
                errors.push((line, format!("past the last cell, {last}")));
            }

            Some(op) => {
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

    // Second pass: the words, now that every label is known.
    let length = statements.iter().map(|statement| statement.address + 1).max();
    let mut memory = vec![0; length.unwrap_or(0)];

    for statement in &statements {
        // Labels first, like MARIE, so a label such as A still works in hex.
        let value = |radix: u32, max: i32| {
            let token = statement.operand.unwrap_or("0");
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

            if (0..=max).contains(&value) {
                Ok(value)
            } else {
                Err(format!("{token} is out of range, 0 to {}", format.text(max, 1)))
            }
        };

        let word = match statement.op {
            Op::Data(radix) => value(radix, max_word),
            Op::Instruction(code, None) => {
                value(format.radix, max_operand).map(|operand| set.join(code, operand))
            }
            Op::Instruction(code, Some(fixed)) => match statement.operand {
                None => Ok(set.join(code, fixed)),
                Some(_) => Err(format!("{} takes no operand", statement.mnemonic)),
            },
            Op::Org => unreachable!("ORG takes no cell"),
        };

        match word {
            Ok(word) => memory[statement.address] = word,
            Err(message) => errors.push((statement.line, message)),
        }
    }

    if errors.is_empty() {
        return Ok(memory);
    }

    errors.sort_by_key(|(line, _)| *line);

    Err(errors
        .into_iter()
        .map(|(line, message)| format!("line {line}: {message}"))
        .collect())
}

// Memory as text that assembles back to the same words, without the trailing zero cells.
pub fn disassemble(memory: &[i32], set: &InstructionSet) -> String {
    let length = memory
        .iter()
        .rposition(|word| *word != 0)
        .map_or(0, |index| index + 1);

    memory[..length]
        .iter()
        .map(|&word| line(word, set) + "\n")
        .collect()
}

// The first instruction that reads back as the word, else the word as data.
fn line(word: i32, set: &InstructionSet) -> String {
    let (code, operand) = set.split(word);
    // Not zero-padded: in hex a padded 0B or 0D would read as a prefix.
    let operand = set.format.text(operand, 1);

    set.definitions
        .iter()
        .filter(|definition| definition.code == code)
        // The bare mnemonic is for an instruction with a fixed operand.
        .flat_map(|definition| {
            [
                format!("{} {operand}", definition.mnemonic),
                definition.mnemonic.clone(),
            ]
        })
        .find(|line| assemble(line, set).is_ok_and(|words| words == [word]))
        .unwrap_or_else(|| format!("DAT {}", set.format.text(word, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::Program;

    fn set(json: &str) -> InstructionSet {
        let set: InstructionSet = serde_json::from_str(json).unwrap();
        set.check().unwrap();
        set
    }

    // Decimal words of one code digit and two operand digits.
    fn lmc() -> InstructionSet {
        set(r#"{"name":"LMC","format":{"radix":10,"code_digits":1,"operand_digits":2},
            "definitions":[{"code":0,"mnemonic":"HLT","opcode":"Halt"},
            {"code":1,"mnemonic":"ADD","opcode":"Add"},
            {"code":3,"mnemonic":"STA","opcode":"Store"},
            {"code":5,"mnemonic":"LDA","opcode":"Load"},
            {"code":6,"mnemonic":"BRA","opcode":"Jump"}]}"#)
    }

    // Hex words of one code digit and three operand digits.
    fn marie() -> InstructionSet {
        set(r#"{"name":"MARIE","format":{"radix":16,"code_digits":1,"operand_digits":3},
            "definitions":[{"code":1,"mnemonic":"LOAD","opcode":"Load"},
            {"code":2,"mnemonic":"STORE","opcode":"Store"},
            {"code":3,"mnemonic":"ADD","opcode":"Add"},
            {"code":7,"mnemonic":"HALT","opcode":"Halt"}]}"#)
    }

    fn round_trip(memory: &[i32], set: &InstructionSet) {
        let text = disassemble(memory, set);
        let length = memory
            .iter()
            .rposition(|word| *word != 0)
            .map_or(0, |index| index + 1);

        assert_eq!(
            assemble(&text, set).unwrap(),
            memory[..length],
            "{}\n{text}",
            set.name
        );
    }

    #[test]
    fn real_programs_round_trip() {
        let countdown: Program =
            serde_json::from_str(include_str!("../assets/countdown.json")).unwrap();
        let sum = Program::from_ram("sum", include_str!("../assets/sum.ram")).unwrap();
        // Adds cell 10 to cell 11 with nothing but TST, JMP, DEC and INC.
        let bonsai = [4010, 3003, 5000, 2010, 1011, 3000, 0, 0, 0, 0, 3, 4];

        round_trip(&countdown.memory, &InstructionSet::default());
        round_trip(&sum.memory, &InstructionSet::johnny());
        round_trip(&bonsai, &InstructionSet::bonsai());
    }

    #[test]
    fn every_kind_of_word_round_trips() {
        let mut unreadable = InstructionSet::default();
        unreadable.definitions[2].mnemonic = "load".to_string();
        unreadable.definitions[3].mnemonic = "DAT".to_string();
        unreadable.definitions[4].mnemonic = "SU B".to_string();
        unreadable.definitions[5].mnemonic = String::new();

        for set in InstructionSet::presets()
            .into_iter()
            .chain([lmc(), marie(), unreadable])
        {
            let words = set.limits().2 + 1;
            // An odd stride reaches every operand and most codes.
            let memory: Vec<i32> = (0..set.format.cells() as i32)
                .map(|address| (address * 7919 + 1) % words)
                .collect();

            round_trip(&memory, &set);
        }
    }

    #[test]
    fn words_of_no_instruction_are_data_and_trailing_zeros_are_left_out() {
        let text = disassemble(&[0x0105, 0x2000, 0x0B0D, 0, 0], &InstructionSet::default());

        assert_eq!(text, "LOAD 5\nDAT 2000\nINC D\n");
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
            assemble(&text, &InstructionSet::default()).unwrap(),
            [0x0106, 0x0A00, 0x0C06, 0x0805, 0x0700, 0xFF00, 5]
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

        assert_eq!(assemble(&text, &lmc()).unwrap(), [504, 104, 305, 601, 1, 0]);
    }

    #[test]
    fn marie_labels_end_in_a_comma_and_org_moves_on() {
        let text = [
            "        ORG 100",
            "        Load X",
            "        Add Y",
            "        Store Z",
            "        Halt",
            "X,      DEC 35",
            "Y,      HEX 1F",
            "Z,      DEC 0",
        ]
        .join("\n");
        let memory = assemble(&text, &marie()).unwrap();

        assert_eq!(memory.len(), 0x107);
        assert!(memory[..0x100].iter().all(|word| *word == 0));
        assert_eq!(memory[0x100..], [0x1104, 0x3105, 0x2106, 0x7000, 35, 0x1F, 0]);
    }

    #[test]
    fn numbers_are_in_the_radix_unless_a_prefix_says_otherwise() {
        let text = "a: DAT 10\nDAT 0x10\nDAT 0d10\nDAT 0b10\nDAT 0B11\nDAT\nLOAD a";

        assert_eq!(
            assemble(text, &InstructionSet::default()).unwrap(),
            [16, 16, 10, 2, 3, 0, 0x0100]
        );
        assert_eq!(
            assemble("DAT 10\nDAT 0x10\nDEC 7", &InstructionSet::johnny()).unwrap(),
            [10, 16, 8007]
        );
    }

    #[test]
    fn every_mistake_is_reported_with_its_line() {
        for (text, error) in [
            ("NOP\nFOO 1", "line 2: unknown mnemonic FOO"),
            ("JMP nowhere", "line 1: unknown label nowhere"),
            ("a: NOP\na: NOP", "line 2: duplicate label a"),
            ("LOAD 100", "line 1: 100 is out of range, 0 to FF"),
            ("DAT 10000", "line 1: 10000 is out of range, 0 to FFFF"),
            ("DAT -1", "line 1: -1 is out of range, 0 to FFFF"),
            ("ORG 100", "line 1: ORG needs an address from 0 to FF"),
            ("ORG FF\nNOP\nNOP", "line 3: past the last cell, FF"),
            ("LOAD 1 2", "line 1: unexpected 2"),
            ("1x: NOP", "line 1: 1x is not a label name"),
            ("DAT 1G", "line 1: 1G is not a number"),
        ] {
            assert_eq!(
                assemble(text, &InstructionSet::default()),
                Err(vec![error.to_string()]),
                "{text}"
            );
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
