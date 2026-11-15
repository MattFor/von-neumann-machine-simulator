use serde::{Deserialize, Serialize};

use super::{ACC_MAX, ACC_MIN, MEMORY_SIZE};

// What a code that no instruction is defined for does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownCode {
    #[default]
    Nop,
    Halt,
}

// What the program counter does when it moves on from the last cell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PcOverflow {
    #[default]
    Wrap,
    Stay,
    Halt,
}

// How signed accumulator values travel through memory on this machine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataEncoding {
    #[default]
    Unsigned,
    TwosComplement,
    Lmc,
}

// What the words of a machine look like and what it does at its edges.
// Fields missing from a file are those of the standard machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Format {
    // Words, codes and operands are shown and read in it.
    pub radix: u32,

    // A word is code * radix^operand_digits + operand.
    pub code_digits: u32,
    pub operand_digits: u32,

    // How many codes and memory cells there are; None is as many as the digits reach.
    pub codes: Option<u32>,
    pub cells: Option<u32>,

    pub acc_min: i32,
    pub acc_max: i32,

    #[serde(default)]
    pub data_encoding: DataEncoding,

    pub unknown_code: UnknownCode,
    pub pc_overflow: PcOverflow,

    // Run stops on an instruction that leaves the program counter where it was.
    pub stop_when_pc_stays: bool,
}

// The standard machine: hex words, a byte of code then a byte of operand.
impl Default for Format {
    fn default() -> Self {
        Self {
            radix: 16,
            code_digits: 2,
            operand_digits: 2,
            codes: None,
            cells: None,
            acc_min: ACC_MIN,
            acc_max: ACC_MAX,
            data_encoding: DataEncoding::Unsigned,
            unknown_code: UnknownCode::Nop,
            pc_overflow: PcOverflow::Wrap,
            stop_when_pc_stays: false,
        }
    }
}

impl Format {
    // Johnny's decimal words: two digits of code, of which only 0 to 19 are used, then three
    // of operand. Like the Johnny simulator it stays on its last cell.
    pub fn johnny() -> Self {
        Self {
            radix: 10,
            operand_digits: 3,
            codes: Some(20),
            pc_overflow: PcOverflow::Stay,
            ..Self::default()
        }
    }

    // Johnny's .ram and .mc files hold decimal words of two code and three operand digits.
    pub fn johnny_compatible(&self) -> bool {
        (self.radix, self.code_digits, self.operand_digits) == (10, 2, 3)
    }

    pub fn lmc() -> Self {
        Self {
            radix: 10,
            code_digits: 1,
            operand_digits: 2,
            codes: Some(10),
            cells: Some(100),
            acc_min: -999,
            acc_max: 999,
            data_encoding: DataEncoding::Lmc,
            ..Self::default()
        }
    }

    pub fn marie() -> Self {
        Self {
            radix: 16,
            code_digits: 1,
            operand_digits: 3,
            codes: Some(16),
            cells: Some(4096),
            acc_min: i16::MIN as i32,
            acc_max: i16::MAX as i32,
            data_encoding: DataEncoding::TwosComplement,
            ..Self::default()
        }
    }

    // How many values that many digits hold.
    pub fn span(&self, digits: u32) -> u32 {
        self.radix.saturating_pow(digits)
    }

    pub fn cells(&self) -> usize {
        self.cells.unwrap_or_else(|| self.span(self.operand_digits)) as usize
    }

    // Zero-padded to `width` digits, letters in upper case like 0x1F.
    pub fn text(&self, value: i32, width: u32) -> String {
        let mut rest = value.unsigned_abs();
        let mut digits = Vec::new();

        while rest > 0 || digits.len() < width.max(1) as usize {
            let digit = char::from_digit(rest % self.radix, self.radix).unwrap_or('?');

            digits.push(digit.to_ascii_uppercase());
            rest /= self.radix;
        }

        if value < 0 {
            digits.push('-');
        }

        digits.iter().rev().collect()
    }

    pub fn code_text(&self, code: i32) -> String {
        self.text(code, self.code_digits)
    }

    // Addresses are written like operands.
    pub fn operand_text(&self, operand: i32) -> String {
        self.text(operand, self.operand_digits)
    }

    pub fn parse(&self, text: &str) -> Option<i32> {
        i32::from_str_radix(text.trim(), self.radix).ok()
    }

    // A word as the number it stands for: MARIE's FFFF is -1.
    pub fn value(&self, word: i32) -> i32 {
        match self.data_encoding {
            DataEncoding::TwosComplement => word as u16 as i16 as i32,
            DataEncoding::Unsigned | DataEncoding::Lmc => word,
        }
    }

    // A number as the word that holds it, so -1 is FFFF on MARIE.
    pub fn word(&self, value: i32) -> i32 {
        match self.data_encoding {
            DataEncoding::TwosComplement => value as u16 as i32,
            DataEncoding::Unsigned | DataEncoding::Lmc => value,
        }
    }

    // What a fetch of this cell leaves in the instruction register: ram->db, then db->ins.
    pub fn fetched(&self, cell: i32) -> i32 {
        let bus = match self.data_encoding {
            DataEncoding::Unsigned => cell.clamp(0, 0xffff),
            DataEncoding::TwosComplement => self.value(cell),
            DataEncoding::Lmc => cell.clamp(-999, 999),
        };

        bus as u16 as i32
    }

    // The registers are 16 bits wide, so every word and address has to fit in them.
    pub fn check(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            (2..=16).contains(&self.radix),
            "the radix must be between 2 and 16"
        );
        anyhow::ensure!(
            self.code_digits > 0 && self.operand_digits > 0,
            "a word needs at least one digit of code and one of operand"
        );
        anyhow::ensure!(
            self.code_digits <= 16 && self.operand_digits <= 16,
            "code and operand digit counts must not exceed 16"
        );

        let code_span = u128::from(self.radix)
            .checked_pow(self.code_digits)
            .ok_or_else(|| anyhow::anyhow!("the code digit count is too large"))?;
        let operand_span = u128::from(self.radix)
            .checked_pow(self.operand_digits)
            .ok_or_else(|| anyhow::anyhow!("the operand digit count is too large"))?;
        let codes = self.codes.map_or(code_span, u128::from);
        let cells = self.cells.map_or(operand_span, u128::from);

        anyhow::ensure!(
            (1..=code_span).contains(&codes),
            "{} code digits make at most {} codes",
            self.code_digits,
            code_span
        );
        anyhow::ensure!(
            operand_span <= 0x1_0000,
            "the operand digit span must fit 16 bits (65536)"
        );
        let largest_word = (codes - 1) * operand_span + (operand_span - 1);
        anyhow::ensure!(
            largest_word <= 0xffff,
            "the largest word must fit 16 bits (65535)"
        );
        anyhow::ensure!(
            (1..=MEMORY_SIZE as u128).contains(&cells),
            "there must be between 1 and {MEMORY_SIZE} cells"
        );
        anyhow::ensure!(
            cells <= operand_span,
            "the operand digits can address at most {operand_span} cells"
        );
        anyhow::ensure!(
            self.acc_min <= self.acc_max,
            "the accumulator minimum is above its maximum"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_shown_and_read_in_the_radix() {
        let octal = Format {
            radix: 8,
            ..Format::default()
        };

        assert_eq!(Format::default().text(0x1f, 2), "1F");
        assert_eq!(Format::johnny().text(12, 3), "012");
        assert_eq!(octal.text(64, 3), "100");
        assert_eq!(octal.text(-8, 3), "-010");
        assert_eq!(octal.text(0, 0), "0");
        assert_eq!(octal.parse(" 777 "), Some(511));
        assert_eq!(octal.parse("8"), None);
    }

    #[test]
    fn words_and_numbers_follow_the_data_encoding() {
        let marie = Format::marie();
        let lmc = Format::lmc();

        assert_eq!(marie.value(0xffff), -1);
        assert_eq!(marie.word(-1), 0xffff);
        assert_eq!(marie.value(0x7fff), 0x7fff);
        assert_eq!(lmc.value(-5), -5);
        assert_eq!(lmc.word(-5), -5);
        assert_eq!(Format::default().word(0x1234), 0x1234);
        assert_eq!(marie.fetched(0x8400), 0x8400);
        assert_eq!(lmc.fetched(-1), 0xffff);
    }

    #[test]
    fn formats_the_registers_cannot_hold_are_refused() {
        assert!(Format::default().check().is_ok());
        assert!(Format::johnny().check().is_ok());

        for format in [
            Format {
                radix: 17,
                ..Format::default()
            },
            Format {
                operand_digits: 3,
                ..Format::default()
            },
            Format {
                code_digits: 0,
                ..Format::default()
            },
            Format {
                codes: Some(101),
                ..Format::johnny()
            },
            Format {
                operand_digits: u32::MAX,
                ..Format::default()
            },
            Format {
                cells: Some(MEMORY_SIZE as u32 + 1),
                ..Format::default()
            },
            Format {
                acc_min: 1,
                acc_max: 0,
                ..Format::default()
            },
        ] {
            assert!(format.check().is_err(), "{format:?}");
        }
    }

    #[test]
    fn cells_must_fit_in_the_operand_address_space() {
        let format = Format {
            cells: Some(1001),
            ..Format::johnny()
        };

        assert!(format.check().is_err());
    }

    #[test]
    fn explicit_code_counts_determine_the_largest_word() {
        let format = Format {
            radix: 10,
            code_digits: 5,
            operand_digits: 1,
            codes: Some(10),
            ..Format::default()
        };

        assert!(format.check().is_ok());

        let too_wide = Format {
            operand_digits: 4,
            ..format
        };
        let error = too_wide.check().unwrap_err().to_string();

        assert!(error.contains("largest word must fit 16 bits"));
    }

    #[test]
    fn enormous_digit_counts_are_refused_without_building_their_spans() {
        let format = Format {
            radix: 2,
            code_digits: u32::MAX,
            codes: Some(1),
            ..Format::default()
        };

        assert!(format.check().is_err());
    }
}
