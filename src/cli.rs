use std::fmt::Display;
use std::io::{self, ErrorKind, Write};
use std::path::Path;

use anyhow::Context;

use crate::machine::{InstructionSet, Machine, Stop};
use crate::program::{Program, has_extension, is_ram};

const HELP: &str = "\
Usage: von-neumann-machine-simulator [run <file> [options]]
       von-neumann-machine-simulator convert <in> <out> [--machine <name>]

Without arguments the window opens. `run` runs a program (.json, .ram, .asm or .mas) without it,
to a halt or the step limit, and prints its output to stdout, one value per line. `convert`
loads a program and saves it in the format of <out>'s extension.

Options:
  --machine <name>  standard, johnny, bonsai, lmc, marie, cardiac or sap1. Default: the set in
                    the file, Johnny for .ram. An .asm file needs it; a .ram file
                    runs on johnny or bonsai, a .mas file always on MARIE
  --input <values>  numbers for IN to read, separated by commas: --input 1,2,-3. 0x and 0b
                    are words in the machine's encoding, so 0xFFFF is -1 on MARIE
  --max-steps <n>   how many instructions to run at most (default 100000)
  --trace           print every instruction to stderr: address, word, mnemonic, accumulator
  -h, --help        print this

Exit codes: 0 halted or converted, 1 bad arguments or file, 2 step limit reached,
3 the program waited for input and none was left, 141 the reader of stdout or stderr went
away, as `| head` does. A run says which on stderr.
";

// In the order of InstructionSet::presets.
const MACHINES: [&str; 7] = ["standard", "johnny", "bonsai", "lmc", "marie", "cardiac", "sap1"];

struct Options {
    convert: bool,
    paths: Vec<String>,
    set: Option<InstructionSet>,
    input: Vec<String>,
    max_steps: usize,
    trace: bool,
}

// The exit code of a run from the command line.
pub fn main(args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print(io::stdout(), HELP.trim_end());

        return 0;
    }

    let result = parse(args).and_then(|options| {
        if options.convert {
            convert(options)
        } else {
            run(options)
        }
    });

    match result {
        Ok(code) => code,

        Err(error) => {
            print(io::stderr(), format_args!("error: {error:#}\nsee --help"));

            1
        }
    }
}

fn parse(args: &[String]) -> anyhow::Result<Options> {
    let mut args = args.iter();

    let convert = match args.next().map(String::as_str) {
        Some("run") => false,
        Some("convert") => true,
        _ => anyhow::bail!("the commands are run and convert"),
    };
    let (files, usage) = if convert {
        (2, "convert <in> <out>")
    } else {
        (1, "run <file>")
    };

    let mut options = Options {
        convert,
        paths: Vec::new(),
        set: None,
        input: Vec::new(),
        max_steps: 100_000,
        trace: false,
    };

    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| anyhow::anyhow!("{arg} needs a value"))
        };

        match arg.as_str() {
            "--machine" => {
                let name = value()?;
                let set = MACHINES
                    .into_iter()
                    .zip(InstructionSet::presets())
                    .find(|(machine, _)| machine.eq_ignore_ascii_case(name))
                    .ok_or_else(|| anyhow::anyhow!("unknown machine {name}"))?;

                options.set = Some(set.1);
            }

            // Read once the machine is known, since its encoding decides what 0xFFFF is.
            "--input" if !convert => {
                options.input = value()?.split(',').map(str::to_string).collect();
            }

            "--max-steps" if !convert => {
                let raw = value()?;

                options.max_steps = raw
                    .parse()
                    .with_context(|| format!("--max-steps {raw} is not a step count"))?;
            }

            "--trace" if !convert => options.trace = true,

            flag if flag.starts_with('-') => anyhow::bail!("unknown option {flag}"),

            file if options.paths.len() < files => options.paths.push(file.to_string()),

            extra => anyhow::bail!("{extra} is one file too many: {usage}"),
        }
    }

    anyhow::ensure!(options.paths.len() == files, "a file is missing: {usage}");

    Ok(options)
}

// The program in `file` on --machine, else on the set it carries, Johnny for .ram.
fn load(file: &str, set: Option<InstructionSet>) -> anyhow::Result<Program> {
    let path = Path::new(file);

    let set = match set {
        Some(set) if has_extension(path, "mas") && set != InstructionSet::marie() => {
            anyhow::bail!("a .mas file is always MARIE, so --machine can only be marie")
        }
        Some(set) if is_ram(path) && !set.format.johnny_compatible() => anyhow::bail!(
            "a .ram file holds Johnny's words, so --machine can only be johnny or bonsai"
        ),
        Some(set) => Some(set),
        None if is_ram(path) => Some(InstructionSet::johnny()),
        None if has_extension(path, "asm") => {
            anyhow::bail!("an .asm file is in one machine's mnemonics, say which with --machine")
        }
        None => None,
    };

    let mut program = Program::load(path, set.as_ref().unwrap_or(&InstructionSet::default()))
        .with_context(|| file.to_string())?;
    program.instruction_set = set.or(program.instruction_set);

    // Checked again on the machine that runs it, which --machine may have changed.
    program.check().with_context(|| file.to_string())?;

    Ok(program)
}

fn convert(options: Options) -> anyhow::Result<i32> {
    let out = &options.paths[1];

    load(&options.paths[0], options.set)?
        .save(Path::new(out))
        .with_context(|| out.clone())?;

    Ok(0)
}

fn run(options: Options) -> anyhow::Result<i32> {
    let mut machine = Machine::new();
    load(&options.paths[0], options.set)?.apply(&mut machine);

    let format = machine.instruction_set.format;

    for raw in &options.input {
        let value = format
            .parse_input(raw)
            .ok_or_else(|| anyhow::anyhow!("{raw} is not a number or a word of this machine"))?;

        machine.push_input(value);
    }

    // The instruction's line first, then what it printed, such as its error.
    let stop = machine.run(options.max_steps, |machine, address, instruction| {
        if options.trace {
            let address = format.operand_text(address as i32);
            let acc = machine.cpu.acc;

            print(io::stderr(), format_args!("{address}  {instruction}  acc {acc}"));
        }

        print_output(machine);
    });

    print_output(&mut machine);

    let at = format.operand_text(machine.cpu.pc as i32);
    let limit = options.max_steps;
    let (code, reason) = match stop {
        Stop::Halted if machine.halted => (0, format!("halted at {at}")),
        Stop::Halted => (0, format!("stopped at {at}, which keeps the program counter")),
        Stop::StepLimit => (2, format!("stopped at {at}: reached the step limit of {limit}")),
        Stop::NoInput => (3, format!("stopped at {at}: waiting for input and none is left")),
    };

    print(io::stderr(), reason);

    Ok(code)
}

// The machine's errors share its console with the values; only the values go to stdout.
fn print_output(machine: &mut Machine) {
    for line in std::mem::take(&mut machine.output).lines() {
        if line.starts_with("error:") {
            print(io::stderr(), line);
        } else {
            print(io::stdout(), line);
        }
    }
}

// A reader that went away, as `| head` does, ends the run with the 128 + SIGPIPE of other tools.
// Any other failure, like a full disk, is said on stderr if that still works.
fn print(mut out: impl Write, line: impl Display) {
    if let Err(error) = writeln!(out, "{line}") {
        if error.kind() == ErrorKind::BrokenPipe {
            std::process::exit(141);
        }

        let _ = writeln!(io::stderr(), "error: failed printing: {error}");
        std::process::exit(1);
    }
}
