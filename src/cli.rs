use std::path::Path;

use anyhow::Context;

use crate::machine::{InstructionSet, Machine, Stop};
use crate::program::{Program, has_extension, is_ram};
use crate::utilities::parse_value;

const HELP: &str = "\
Usage: von-neumann-machine-simulator [run <file> [options]]

Without arguments the window opens. `run` runs a program (.json, .ram or .asm) without it,
to a halt or the step limit, and prints its output to stdout, one value per line.

Options:
  --machine <name>  standard, johnny, bonsai, lmc or marie. Default: the set in the file,
                    Johnny for .ram. An .asm file needs it
  --input <values>  numbers for IN to read, separated by commas: --input 1,2,3
  --max-steps <n>   how many instructions to run at most (default 100000)
  --trace           print every instruction to stderr: address, word, mnemonic, accumulator
  -h, --help        print this

Exit codes: 0 halted, 1 bad arguments or file, 2 step limit reached,
3 the program waited for input and none was left. Which one is said on stderr.
";

// In the order of InstructionSet::presets.
const MACHINES: [&str; 5] = ["standard", "johnny", "bonsai", "lmc", "marie"];

struct Options {
    path: String,
    set: Option<InstructionSet>,
    input: Vec<i32>,
    max_steps: usize,
    trace: bool,
}

// The exit code of a run from the command line.
pub fn main(args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print!("{HELP}");

        return 0;
    }

    match parse(args).and_then(run) {
        Ok(code) => code,

        Err(error) => {
            eprintln!("error: {error:#}\nsee --help");

            1
        }
    }
}

fn parse(args: &[String]) -> anyhow::Result<Options> {
    let mut args = args.iter();

    anyhow::ensure!(
        args.next().is_some_and(|command| command == "run"),
        "the only command is run"
    );

    let mut path = None;
    let mut options = Options {
        path: String::new(),
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

            "--input" => {
                options.input = value()?
                    .split(',')
                    .map(|raw| {
                        parse_value(raw)
                            .ok_or_else(|| anyhow::anyhow!("{raw} is not a number"))
                    })
                    .collect::<anyhow::Result<_>>()?;
            }

            "--max-steps" => {
                let raw = value()?;

                options.max_steps = raw
                    .parse()
                    .with_context(|| format!("--max-steps {raw} is not a step count"))?;
            }

            "--trace" => options.trace = true,

            flag if flag.starts_with('-') => anyhow::bail!("unknown option {flag}"),

            file if path.is_none() => path = Some(file.to_string()),

            extra => anyhow::bail!("one file only, not also {extra}"),
        }
    }

    options.path = path.ok_or_else(|| anyhow::anyhow!("run needs a program file"))?;

    Ok(options)
}

fn run(options: Options) -> anyhow::Result<i32> {
    let path = Path::new(&options.path);

    let set = match options.set {
        Some(set) => Some(set),
        None if is_ram(path) => Some(InstructionSet::johnny()),
        None if has_extension(path, "asm") => {
            anyhow::bail!("an .asm file is in one machine's mnemonics, say which with --machine")
        }
        None => None,
    };

    let mut program = Program::load(path, set.as_ref().unwrap_or(&InstructionSet::default()))
        .with_context(|| options.path.clone())?;
    program.instruction_set = set.or(program.instruction_set);

    let mut machine = Machine::new();
    program.apply(&mut machine);

    for value in options.input {
        machine.push_input(value);
    }

    let stop = machine.run(options.max_steps, |machine, address, instruction| {
        print_output(machine);

        if options.trace {
            let address = machine.instruction_set.format.operand_text(address as i32);

            eprintln!("{address}  {instruction}  acc {}", machine.cpu.acc);
        }
    });

    print_output(&mut machine);

    let format = machine.instruction_set.format;
    let at = format.operand_text(machine.cpu.pc as i32);
    let limit = options.max_steps;
    let (code, reason) = match stop {
        Stop::Halted if machine.halted => (0, format!("halted at {at}")),
        Stop::Halted => (0, format!("stopped at {at}, which keeps the program counter")),
        Stop::StepLimit => (2, format!("stopped at {at}: reached the step limit of {limit}")),
        Stop::NoInput => (3, format!("stopped at {at}: waiting for input and none is left")),
    };

    eprintln!("{reason}");

    Ok(code)
}

// The machine's errors share its console with the values; only the values go to stdout.
fn print_output(machine: &mut Machine) {
    for line in std::mem::take(&mut machine.output).lines() {
        if line.starts_with("error:") {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
}
