use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::assembler::{as_read, assemble, disassemble};
use crate::machine::{DataEncoding, InstructionSet, MEMORY_SIZE, Machine};
use crate::utilities::read_text;

pub fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

pub fn is_ram(path: &Path) -> bool {
    has_extension(path, "ram")
}

// The first mistake is enough for the status bar.
fn assembled(text: &str, set: &InstructionSet) -> anyhow::Result<(Vec<i32>, u16)> {
    assemble(text, set).map_err(|errors| match errors.len() {
        1 => anyhow::anyhow!("{}", errors[0]),
        count => anyhow::anyhow!("{}, and {} more", errors[0], count - 1),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub name: String,

    pub entry_point: u16,

    pub memory: Vec<i32>,

    // Absent in older files; those keep whatever set is loaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_set: Option<InstructionSet>,
}

fn validate_program(program: &Program, set: &InstructionSet) -> anyhow::Result<()> {
    set.check()?;

    let format = &set.format;
    let cells = format.cells();

    anyhow::ensure!(
        usize::from(program.entry_point) < cells,
        "the program starts at {}, past the last cell, {}",
        format.text(program.entry_point.into(), 1),
        format.text(cells as i32 - 1, 1)
    );

    if format.data_encoding == DataEncoding::Lmc {
        anyhow::ensure!(
            program.memory.len() <= cells
                && program.memory.iter().all(|word| (-999..=999).contains(word)),
            "an LMC program has at most {cells} cells, each between -999 and 999"
        );

        return Ok(());
    }

    anyhow::ensure!(
        program.memory.len() <= MEMORY_SIZE
            && program
                .memory
                .iter()
                .all(|word| u16::try_from(*word).is_ok()),
        "a program is at most {MEMORY_SIZE} words, each between 0 and 65535"
    );

    Ok(())
}

fn validate_ram_program(program: &Program, set: &InstructionSet) -> anyhow::Result<()> {
    validate_program(program, set)?;
    anyhow::ensure!(
        program.memory.len() <= 1000,
        "a .ram file has at most 1000 cells"
    );

    Ok(())
}

pub struct Example {
    pub name: &'static str,
    pub preset: InstructionSet,
    pub program: Program,
}

pub fn examples() -> Vec<Example> {
    fn example(
        name: &'static str,
        preset: InstructionSet,
        mut program: Program,
    ) -> Example {
        program
            .instruction_set
            .get_or_insert_with(|| preset.clone());

        Example {
            name,
            preset,
            program,
        }
    }

    vec![
        example(
            "Standard countdown",
            InstructionSet::default(),
            serde_json::from_str(include_str!("../assets/countdown.json"))
                .expect("bundled Standard example is valid"),
        ),
        example(
            "Johnny sum",
            InstructionSet::johnny(),
            Program::from_ram("sum", include_str!("../assets/sum.ram"))
                .expect("bundled Johnny example is valid"),
        ),
        example(
            "Bonsai increment",
            InstructionSet::bonsai(),
            serde_json::from_str(include_str!("../assets/bonsai.json"))
                .expect("bundled Bonsai example is valid"),
        ),
        example(
            "LMC echo",
            InstructionSet::lmc(),
            serde_json::from_str(include_str!("../assets/lmc-echo.json"))
                .expect("bundled LMC example is valid"),
        ),
        example(
            "MARIE echo",
            InstructionSet::marie(),
            serde_json::from_str(include_str!("../assets/marie-echo.json"))
                .expect("bundled MARIE example is valid"),
        ),
        example(
            "CARDIAC digits",
            InstructionSet::cardiac(),
            Program::from_asm(
                "digits",
                include_str!("../assets/cardiac-digits.asm"),
                &InstructionSet::cardiac(),
            )
            .expect("bundled CARDIAC example is valid"),
        ),
        example(
            "SAP-1 count",
            InstructionSet::sap1(),
            Program::from_asm(
                "count",
                include_str!("../assets/sap1-count.asm"),
                &InstructionSet::sap1(),
            )
            .expect("bundled SAP-1 example is valid"),
        ),
    ]
}

impl Program {
    // What the machine reads, so a fixed cell holds its word.
    pub fn from_machine(name: String, machine: &Machine) -> Self {
        let data: Vec<i32> = (0..MEMORY_SIZE).map(|address| machine.read(address)).collect();

        let length = data
            .iter()
            .rposition(|value| *value != 0)
            .map_or(0, |index| index + 1);

        Self {
            name,
            entry_point: machine.entry_point,
            memory: data[..length].to_vec(),
            instruction_set: Some(machine.instruction_set.clone()),
        }
    }

    // A Johnny program file: one decimal word per line.
    pub fn from_ram(name: &str, text: &str) -> anyhow::Result<Self> {
        let memory = text
            .lines()
            .map(|line| match line.trim() {
                "" => Ok(0),
                word => word.parse(),
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            name: name.to_string(),
            entry_point: 0,
            memory,
            instruction_set: None,
        })
    }

    // Johnny has exactly 1000 cells.
    pub fn to_ram(&self) -> String {
        let words: Vec<String> = (0..1000)
            .map(|address| self.memory.get(address).copied().unwrap_or(0).to_string())
            .collect();

        words.join("\r\n")
    }

    pub fn from_asm(name: &str, text: &str, set: &InstructionSet) -> anyhow::Result<Self> {
        let (memory, entry_point) = assembled(text, set)?;

        Ok(Self {
            name: name.to_string(),
            entry_point,
            memory,
            instruction_set: None,
        })
    }

    // Whether the program fits the set it will run on: its own, else that of the machine it
    // lands on.
    pub fn check_on(&self, machine: &InstructionSet) -> anyhow::Result<()> {
        validate_program(self, self.instruction_set.as_ref().unwrap_or(machine))
    }

    // A program without a set runs as Standard.
    pub fn check(&self) -> anyhow::Result<()> {
        self.check_on(&InstructionSet::default())
    }

    // Files ending in .ram are Johnny's words without a set, .asm are text in the mnemonics of
    // `set`, .mas are MARIE's, everything else is JSON. `set` is the machine's it lands on, so a
    // file without a set of its own is checked against it.
    pub fn load(path: &Path, set: &InstructionSet) -> anyhow::Result<Self> {
        let text = read_text(path)?;
        let name = path.file_stem().unwrap_or_default().to_string_lossy();

        let program: Self = if is_ram(path) {
            Self::from_ram(&name, &text)?
        } else if has_extension(path, "asm") {
            Self::from_asm(&name, &text, set)?
        } else if has_extension(path, "mas") {
            let marie = InstructionSet::marie();

            Self {
                instruction_set: Some(marie.clone()),
                ..Self::from_asm(&name, &text, &marie)?
            }
        } else {
            serde_json::from_str(&text)?
        };

        if is_ram(path) {
            validate_ram_program(&program, set)?;
        } else {
            program.check_on(set)?;
        }

        Ok(program)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        // A program without a set runs as Standard, so it is saved as one.
        let standard = InstructionSet::default();
        let own = self.instruction_set.as_ref().unwrap_or(&standard);

        let text = if is_ram(path) {
            // Johnny reads decimal words, so another format would come back as another program.
            anyhow::ensure!(
                own.format.johnny_compatible(),
                "a .ram file holds Johnny's words: radix 10, 2 code digits and 3 operand digits"
            );

            anyhow::ensure!(
                self.entry_point == 0,
                "a .ram file has no entry point, so the program must start at 0"
            );

            // Only words travel, as in Johnny, so they are checked the way a load checks them.
            validate_ram_program(self, &InstructionSet::johnny())?;

            self.to_ram()
        } else if has_extension(path, "asm") || has_extension(path, "mas") {
            let marie = InstructionSet::marie();
            let set = if has_extension(path, "mas") {
                // .mas files always load as MARIE programs, so another set would be lost.
                anyhow::ensure!(
                    own.format == marie.format,
                    "a .mas file holds MARIE's words: 16-bit hex, 1 code digit and 3 operand digits"
                );
                anyhow::ensure!(
                    own.definitions == marie.definitions,
                    "a .mas file loads with MARIE's instructions, so save {} as .json or .asm",
                    own.name
                );

                &marie
            } else {
                own
            };
            validate_program(self, set)?;
            let text = disassemble(&self.memory, self.entry_point, set);

            // Words or cells the format cannot hold would not load back.
            let (memory, entry_point) = assembled(&text, set)?;
            let format = &set.format;

            anyhow::ensure!(
                (as_read(&memory, format), entry_point)
                    == (as_read(&self.memory, format), self.entry_point),
                "this program would not read back the same from an .asm file"
            );

            text
        } else {
            self.check()?;
            serde_json::to_string_pretty(self)?
        };

        fs::write(path, text)?;

        Ok(())
    }

    pub fn apply(&self, machine: &mut Machine) {
        machine.memory.load(&self.memory);
        machine.entry_point = self.entry_point;
        machine.breakpoints.clear();

        if let Some(set) = &self.instruction_set {
            machine.instruction_set = set.clone();
        }

        machine.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{Format, PcOverflow};
    use crate::utilities::temp_path;

    // A file of its own per test, since tests run in parallel.
    fn load(name: &str, text: &str) -> anyhow::Result<Program> {
        let path = temp_path(name);
        fs::write(&path, text).unwrap();

        let program = Program::load(&path, &InstructionSet::default());
        fs::remove_file(&path).unwrap();

        program
    }

    #[test]
    fn words_that_do_not_fit_sixteen_bits_are_refused() {
        for memory in ["[-1]", "[70000]"] {
            let text = format!(r#"{{"name":"x","entry_point":0,"memory":{memory}}}"#);

            assert!(load("word.json", &text).is_err());
        }

        assert!(load("negative.ram", "1010\r\n-5\r\n").is_err());
        assert!(load("fits.ram", "1010\r\n65535\r\n").is_ok());
    }

    #[test]
    fn programs_larger_than_the_memory_are_refused() {
        assert!(load("long.ram", &"0\n".repeat(MEMORY_SIZE + 1)).is_err());
    }

    #[test]
    fn instruction_codes_out_of_range_are_refused() {
        let text = r#"{"name":"x","entry_point":0,"memory":[],"instruction_set":
            {"name":"s","definitions":[{"code":300,"mnemonic":"X","opcode":"Nop"}]}}"#;

        assert!(load("code.json", text).is_err());
    }

    #[test]
    fn upper_case_ram_files_are_read_as_johnny_programs() {
        let program = load("upper.RAM", "1010\r\n2011\r\n").unwrap();

        assert_eq!(program.memory, [1010, 2011]);
    }

    #[test]
    fn only_a_johnny_program_is_saved_as_ram() {
        let path = temp_path("standard.ram");
        let mut machine = Machine::new();

        let saved = Program::from_machine("x".to_string(), &machine).save(&path);
        let written = fs::remove_file(&path).is_ok();

        assert!(saved.is_err() && !written);

        machine.instruction_set = InstructionSet::johnny();
        Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap();
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn ram_files_need_a_johnny_compatible_format() {
        let path = temp_path("compatible.ram");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        machine.instruction_set.format.pc_overflow = PcOverflow::Halt;

        Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap();
        fs::remove_file(&path).unwrap();

        machine.instruction_set.format.radix = 8;
        let error = Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap_err();

        assert!(error.to_string().contains("radix 10"));
    }

    // Like Johnny's own simulator, any set of Johnny's words saves its words.
    #[test]
    fn bonsai_and_microcode_programs_are_saved_as_ram() {
        let path = temp_path("bonsai.ram");
        let microcode = InstructionSet::from_mc("mine", &"0\n".repeat(200)).unwrap();

        for set in [InstructionSet::bonsai(), microcode] {
            let mut machine = Machine::new();
            machine.instruction_set = set;
            machine.memory.load(&[1005, 5000]);

            Program::from_machine("x".to_string(), &machine)
                .save(&path)
                .unwrap();
            let loaded = Program::load(&path, &machine.instruction_set).unwrap();
            fs::remove_file(&path).unwrap();

            assert_eq!(loaded.memory[..2], [1005, 5000]);
        }
    }

    #[test]
    fn a_mas_file_does_not_drop_a_changed_marie_set() {
        let path = temp_path("changed.mas");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::marie();
        machine.instruction_set.definitions[1].mnemonic = "Fetch".to_string();
        machine.memory.load(&[0x1000]);

        let error = Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap_err();
        let written = fs::remove_file(&path).is_ok();

        assert!(error.to_string().contains("MARIE's instructions"), "{error}");
        assert!(!written);
    }

    #[test]
    fn an_entry_point_past_the_last_cell_is_refused_on_load_and_save() {
        let program = Program {
            name: "x".to_string(),
            entry_point: 0x12C,
            memory: vec![0x0105],
            instruction_set: Some(InstructionSet::default()),
        };
        let path = temp_path("far-entry.asm");

        let loaded = load("far-entry.json", &serde_json::to_string(&program).unwrap());
        let saved = program.save(&path);
        let written = fs::remove_file(&path).is_ok();

        for error in [loaded.unwrap_err(), saved.unwrap_err()] {
            assert_eq!(error.to_string(), "the program starts at 12C, past the last cell, FF");
        }
        assert!(!written);
    }

    #[test]
    fn a_ram_file_cannot_start_past_cell_0() {
        let path = temp_path("entry.ram");
        let program = Program {
            name: "x".to_string(),
            entry_point: 2,
            memory: vec![1010],
            instruction_set: Some(InstructionSet::johnny()),
        };

        let error = program.save(&path).unwrap_err();
        let written = fs::remove_file(&path).is_ok();

        assert!(error.to_string().contains("must start at 0"), "{error}");
        assert!(!written);
    }

    #[test]
    fn ram_saves_validate_using_the_set_that_load_will_use() {
        let path = temp_path("lmc-data.ram");
        let mut set = InstructionSet::johnny();
        set.format.data_encoding = DataEncoding::Lmc;
        let program = Program {
            name: "signed".to_string(),
            entry_point: 0,
            memory: vec![5, -5],
            instruction_set: Some(set),
        };

        let save_error = program.save(&path).unwrap_err().to_string();
        fs::write(&path, "5\r\n-5\r\n").unwrap();
        let load_error = Program::load(&path, &InstructionSet::johnny())
            .unwrap_err()
            .to_string();
        fs::remove_file(&path).unwrap();

        assert_eq!(save_error, load_error);
    }

    #[test]
    fn ram_files_cannot_silently_drop_cells_past_johnnys_memory() {
        let error = load("too-many.ram", &"0\n".repeat(1001)).unwrap_err();

        assert!(error.to_string().contains("at most 1000 cells"));
    }

    #[test]
    fn programs_from_before_the_format_load_by_their_johnny_switch() {
        let text = r#"{"name":"x","entry_point":0,"memory":[1010],"instruction_set":
            {"name":"Johnny","johnny":true,"definitions":[]}}"#;
        let program = load("old-program.json", text).unwrap();

        assert_eq!(program.instruction_set.unwrap().format, Format::johnny());
    }

    #[test]
    fn a_saved_program_keeps_its_format() {
        let path = temp_path("format.json");
        let mut machine = Machine::new();
        machine.instruction_set.format = Format {
            radix: 8,
            code_digits: 1,
            operand_digits: 3,
            acc_min: -999,
            pc_overflow: PcOverflow::Halt,
            ..Format::default()
        };
        machine
            .instruction_set
            .definitions
            .retain(|definition| definition.code < 8);

        Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let program = Program::load(&path, &machine.instruction_set).unwrap();
        fs::remove_file(&path).unwrap();

        assert!(text.contains(r#""radix": 8"#));
        assert_eq!(
            program.instruction_set.unwrap().format,
            machine.instruction_set.format
        );
    }

    #[test]
    fn saving_and_loading_share_the_lmc_program_validation() {
        let mut machine = Machine::new();
        let countdown: Program = serde_json::from_str(include_str!("../assets/countdown.json"))
            .unwrap();
        countdown.apply(&mut machine);
        machine.instruction_set = InstructionSet::lmc();

        let program = Program::from_machine("countdown".to_string(), &machine);
        let path = temp_path("countdown-lmc.json");
        let save_error = program.save(&path).unwrap_err().to_string();
        fs::write(&path, serde_json::to_string(&program).unwrap()).unwrap();
        let load_error = Program::load(&path, &InstructionSet::default())
            .unwrap_err()
            .to_string();
        fs::remove_file(&path).unwrap();

        assert_eq!(save_error, load_error);
        assert!(save_error.contains("an LMC program has at most 100 cells"));
    }

    #[test]
    fn bundled_json_examples_embed_their_instruction_sets() {
        for path in [
            Path::new("assets/countdown.json"),
            Path::new("assets/bonsai.json"),
            Path::new("assets/lmc-echo.json"),
            Path::new("assets/marie-echo.json"),
        ] {
            let program = Program::load(path, &InstructionSet::default()).unwrap();

            let set = program.instruction_set.unwrap();
            set.check().unwrap();
        }
    }

    #[test]
    fn huge_files_are_refused_before_they_are_read() {
        let path = temp_path("huge.json");
        fs::File::create(&path)
            .unwrap()
            .set_len(16 * 1024 * 1024 + 1)
            .unwrap();

        let error = Program::load(&path, &InstructionSet::default()).unwrap_err();
        fs::remove_file(&path).unwrap();

        assert!(error.to_string().contains("larger"));
    }

    #[test]
    fn asm_files_are_written_and_read_in_the_loaded_set() {
        let path = temp_path("sum.ASM");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        Program::from_ram("sum", include_str!("../assets/sum.ram"))
            .unwrap()
            .apply(&mut machine);
        machine.entry_point = 2;

        let saved = Program::from_machine("sum".to_string(), &machine);
        saved.save(&path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let program = Program::load(&path, &machine.instruction_set).unwrap();
        fs::remove_file(&path).unwrap();

        assert!(text.starts_with("TAKE 10\nADD 11\nSAVE 12\nHLT 0\n"));
        assert!(text.ends_with("NOP 7\nEND 2\n"));
        assert_eq!(program.memory, saved.memory);
        assert_eq!(program.entry_point, 2);
        assert!(program.instruction_set.is_none());
    }

    #[test]
    fn lmc_asm_files_keep_negative_data() {
        let path = temp_path("negative-lmc.asm");
        let set = InstructionSet::lmc();
        let saved = Program {
            name: "negative".to_string(),
            entry_point: 0,
            memory: vec![901, -5],
            instruction_set: Some(set.clone()),
        };

        saved.save(&path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let loaded = Program::load(&path, &set).unwrap();
        fs::remove_file(&path).unwrap();

        assert!(text.contains("DAT -5\n"));
        assert_eq!(loaded.memory, saved.memory);
        assert_eq!(loaded.entry_point, saved.entry_point);
    }

    #[test]
    fn mas_files_are_marie_programs_whatever_set_is_loaded() {
        let path = temp_path("echo.MAS");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::marie();
        machine.memory.load(&[0x5000, 0x6000, 0x7000, 0xFFE9]);

        Program::from_machine("echo".to_string(), &machine)
            .save(&path)
            .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let program = Program::load(&path, &InstructionSet::lmc()).unwrap();

        assert!(text.starts_with("Input 0\nOutput 0\nHalt 0\nDEC -23\n"));
        assert_eq!(program.memory, [0x5000, 0x6000, 0x7000, 0xFFE9]);
        assert_eq!(program.instruction_set, Some(InstructionSet::marie()));

        machine.instruction_set = InstructionSet::lmc();
        fs::remove_file(&path).unwrap();
        let error = Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap_err();
        let written = fs::remove_file(&path).is_ok();

        assert!(error.to_string().contains("MARIE's words"));
        assert!(!written);
    }

    #[test]
    fn a_fixed_cell_is_saved_as_its_word() {
        let [json, asm] = ["json", "asm"].map(|extension| temp_path(&format!("fixed.{extension}")));
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::cardiac();
        machine.memory.load(&[5, 900]);
        machine.entry_point = 1;

        let saved = Program::from_machine("x".to_string(), &machine);
        saved.save(&json).unwrap();
        saved.save(&asm).unwrap();
        let text = fs::read_to_string(&asm).unwrap();
        let loaded = [&json, &asm].map(|path| Program::load(path, &machine.instruction_set));
        fs::remove_file(&json).unwrap();
        fs::remove_file(&asm).unwrap();

        assert_eq!(saved.memory, [1, 900]);
        assert_eq!(text, "ORG 1\nHRS 0\n");
        assert_eq!(loaded.map(|program| program.unwrap().memory), [[1, 900], [1, 900]]);
    }

    // A file's raw cell 0 is not what the machine reads there, the loader's word is.
    #[test]
    fn a_cardiac_file_whose_fixed_cell_is_raw_converts_to_asm() {
        let text = r#"{"name":"x","entry_point":1,"memory":[0,900],"instruction_set":"#;
        let set = serde_json::to_string(&InstructionSet::cardiac()).unwrap();
        let program = load("raw-fixed.json", &format!("{text}{set}}}")).unwrap();
        let path = temp_path("raw-fixed.asm");

        program.save(&path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(text, "ORG 1\nHRS 0\n");
    }

    #[test]
    fn asm_files_report_the_first_mistake_and_how_many_more() {
        let error = load("mistakes.asm", "LOAD 1\nFOO\nBAR\n").unwrap_err();

        assert_eq!(error.to_string(), "line 2: unknown mnemonic FOO, and 1 more");
    }

    #[test]
    fn memory_the_format_cannot_hold_is_not_saved_as_asm() {
        let path = temp_path("wide.asm");
        let mut machine = Machine::new();
        machine.instruction_set = InstructionSet::johnny();
        machine.memory.write(0, 65535);

        let error = Program::from_machine("x".to_string(), &machine)
            .save(&path)
            .unwrap_err();
        let written = fs::remove_file(&path).is_ok();

        assert!(error.to_string().starts_with("line 1: 65535 is out of range"));
        assert!(!written);
    }
}
