use std::fs;
use std::process::{Command, Stdio};

const ADD: &str = "tests/programs/lmc-add.asm";

fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_von-neumann-machine-simulator"));
    command.current_dir(env!("CARGO_MANIFEST_DIR"));

    command
}

// stdout, stderr and the exit code of the binary.
fn call(args: &[&str]) -> (String, String, i32) {
    let output = binary().args(args).output().unwrap();

    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.code().unwrap(),
    )
}

fn run(args: &[&str]) -> (String, String, i32) {
    call(&[&["run"], args].concat())
}

fn convert(args: &[&str]) -> (String, String, i32) {
    call(&[&["convert"], args].concat())
}

// A path in the temp directory, as a string for the arguments.
fn temp(name: &str) -> String {
    let path = std::env::temp_dir().join(format!("vnm-cli-{name}"));

    path.to_str().unwrap().to_string()
}

// stdout and the exit code of a program from tests/programs.
fn program(file: &str, machine: &str, input: &str) -> (String, i32) {
    let path = format!("tests/programs/{file}");
    let mut args = vec![path.as_str(), "--machine", machine];

    if !input.is_empty() {
        args.extend(["--input", input]);
    }

    let (stdout, _, code) = run(&args);

    (stdout, code)
}

fn halted(stdout: &str) -> (String, i32) {
    (stdout.to_string(), 0)
}

// Johnny and Bonsai have no output, so the trace shows the result.
fn traced(file: &str, machine: &str) -> String {
    let path = format!("tests/programs/{file}");
    let (stdout, stderr, code) = run(&[&path, "--machine", machine, "--trace"]);

    assert_eq!((stdout.as_str(), code), ("", 0), "{stderr}");

    stderr
}

#[test]
fn lmc_adds_two_inputs() {
    assert_eq!(program("lmc-add.asm", "lmc", "12,30"), halted("42\n"));
}

#[test]
fn lmc_counts_down_to_zero() {
    assert_eq!(
        program("lmc-countdown.asm", "lmc", "5"),
        halted("5\n4\n3\n2\n1\n0\n")
    );
}

#[test]
fn lmc_multiplies_by_repeated_addition() {
    assert_eq!(program("lmc-multiply.asm", "lmc", "6,7"), halted("42\n"));
    assert_eq!(program("lmc-multiply.asm", "lmc", "6,0"), halted("0\n"));
}

#[test]
fn lmc_outputs_the_larger_input() {
    assert_eq!(program("lmc-max.asm", "lmc", "17,42"), halted("42\n"));
    assert_eq!(program("lmc-max.asm", "lmc", "42,17"), halted("42\n"));
    assert_eq!(program("lmc-max.asm", "lmc", "9,9"), halted("9\n"));
}

#[test]
fn lmc_divides_until_the_rest_goes_negative() {
    assert_eq!(program("lmc-divide.asm", "lmc", "17,5"), halted("3\n2\n"));
    assert_eq!(program("lmc-divide.asm", "lmc", "15,5"), halted("3\n0\n"));
}

#[test]
fn marie_adds_two_inputs() {
    assert_eq!(program("marie-add.asm", "marie", "12,30"), halted("42\n"));
}

#[test]
fn marie_sums_one_to_n() {
    assert_eq!(program("marie-sum.asm", "marie", "10"), halted("55\n"));
    assert_eq!(program("marie-sum.asm", "marie", "0"), halted("0\n"));
}

#[test]
fn marie_subroutine_returns_to_each_caller() {
    assert_eq!(
        program("marie-subroutine.asm", "marie", "21"),
        halted("42\n84\n")
    );
}

#[test]
fn marie_outputs_twos_complement_negatives() {
    assert_eq!(
        program("marie-negative.asm", "marie", ""),
        halted("-7\n-2\n-9\n")
    );
}

#[test]
fn standard_prints_fibonacci_numbers() {
    assert_eq!(
        program("standard-fibonacci.asm", "standard", "0xA"),
        halted("0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n")
    );
}

#[test]
fn sap1_multiplies_with_the_carry_flag() {
    assert_eq!(program("sap1-multiply.asm", "sap1", ""), halted("42\n"));
}

#[test]
fn johnny_multiplies_into_the_accumulator() {
    let trace = traced("johnny-multiply.asm", "johnny");

    assert!(
        trace.ends_with("009  10.000  HLT 000  acc 42\nhalted at 009\n"),
        "{trace}"
    );
}

#[test]
fn bonsai_adds_with_inc_and_dec() {
    let trace = traced("bonsai-add.asm", "bonsai");
    let incs: Vec<&str> = trace.lines().filter(|line| line.contains("INC")).collect();

    assert_eq!(incs.len(), 4, "{trace}");
    assert_eq!(incs[3], "004  01.006  INC 006  acc 7");
    assert!(trace.ends_with("halted at 002\n"), "{trace}");
}

#[test]
fn files_bring_their_own_machine() {
    assert_eq!(run(&["assets/countdown.json"]).0, "5\n4\n3\n2\n1\n");

    let (_, stderr, code) = run(&["assets/sum.ram", "--trace"]);

    assert_eq!(code, 0);
    assert!(stderr.contains("002  04.012  SAVE 012  acc 12"), "{stderr}");
}

#[test]
fn bad_arguments_and_files_exit_with_1() {
    for args in [
        &[ADD][..],
        &[ADD, "--machine", "pdp11"],
        &[ADD, "--machine", "lmc", "--input", "1,x"],
        &[ADD, "--machine", "lmc", "--input", "0x10000"],
        &[ADD, ADD, "--machine", "lmc"],
        &[ADD, "--machine", "lmc", "--max-steps"],
        &[ADD, "--machine", "lmc", "--fast"],
        &[ADD, "--machine", "marie"],
        &["tests/programs/missing.asm", "--machine", "lmc"],
        &[],
    ] {
        let (stdout, stderr, code) = run(args);

        assert_eq!((stdout.as_str(), code), ("", 1), "{args:?}");
        assert!(stderr.starts_with("error: "), "{args:?}: {stderr}");
    }

    assert!(run(&[ADD]).1.contains("--machine"));
    assert_eq!(call(&["frob"]).2, 1);
}

#[test]
fn input_words_are_read_in_the_machines_encoding() {
    assert_eq!(program("marie-add.asm", "marie", "0xFFFF,0b10"), halted("1\n"));
    assert_eq!(program("marie-add.asm", "marie", "-1, 2"), halted("1\n"));
    assert_eq!(program("lmc-add.asm", "lmc", "0x10,1"), halted("17\n"));
}

// What `| head` does: the reader goes away while the program still prints.
#[test]
fn a_closed_pipe_exits_with_141_without_a_panic() {
    let path = temp("forever.asm");
    fs::write(&path, "loop OUT\n BRA loop\n").unwrap();

    let args = ["run", &path, "--machine", "lmc", "--max-steps", "1000000"];
    let mut child = binary()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();

    let mut traced = binary()
        .args(args)
        .arg("--trace")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(traced.stderr.take());
    let traced = traced.wait().unwrap();
    fs::remove_file(&path).unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!((output.status.code(), stderr.as_str()), (Some(141), ""));
    assert_eq!(traced.code(), Some(141));
}

// Help and errors fit in the pipe, so its reader has to be gone before they are printed.
#[test]
fn help_and_errors_exit_with_141_on_a_closed_pipe() {
    let closed = || {
        let (reader, writer) = std::io::pipe().unwrap();
        drop(reader);

        writer
    };

    let help = binary().arg("--help").stdout(closed()).status().unwrap();
    let error = binary().arg("frob").stderr(closed()).status().unwrap();

    assert_eq!((help.code(), error.code()), (Some(141), Some(141)));
}

// As `run` does, convert reads a JSON program without a set as a Standard one.
#[test]
fn a_json_program_without_a_set_converts_as_standard() {
    let [json, ram, mas, asm] =
        ["json", "ram", "mas", "asm"].map(|extension| temp(&format!("setless.{extension}")));
    fs::write(
        &json,
        r#"{"name":"countdown","entry_point":0,"memory":[266,2560,3082,266,2054,1793,65280,0,0,0,5]}"#,
    )
    .unwrap();

    let refused = [&ram, &mas].map(|out| convert(&[&json, out]));
    let made = convert(&[&json, &asm]);
    let runs = [run(&[&json]).0, run(&[&asm, "--machine", "standard"]).0];

    for path in [&json, &asm] {
        fs::remove_file(path).unwrap();
    }

    let messages = ["radix 10", "MARIE's words"];

    for ((stdout, stderr, code), message) in refused.into_iter().zip(messages) {
        assert_eq!((stdout.as_str(), code), ("", 1), "{stderr}");
        assert!(stderr.contains(message), "{stderr}");
    }

    assert_eq!(made.2, 0, "{}", made.1);
    assert_eq!(runs, ["5\n4\n3\n2\n1\n"; 2]);
}

#[test]
fn convert_moves_a_program_between_formats() {
    let (asm, json, ram) = (temp("sum.asm"), temp("sum.json"), temp("sum.ram"));

    for args in [
        &["assets/sum.ram", &asm][..],
        &[&asm, &json, "--machine", "johnny"],
        &[&json, &ram],
    ] {
        assert_eq!(convert(args), (String::new(), String::new(), 0), "{args:?}");
    }

    let text = fs::read_to_string(&asm).unwrap();
    let trace = run(&[&ram, "--trace"]);

    for path in [&asm, &json, &ram] {
        fs::remove_file(path).unwrap();
    }

    assert!(text.starts_with("TAKE 10\nADD 11\nSAVE 12\nHLT 0\n"), "{text}");
    assert_eq!(trace, run(&["assets/sum.ram", "--trace"]));
}

#[test]
fn converted_marie_programs_run_from_mas_and_json() {
    let (mas, json) = (temp("add.mas"), temp("add.json"));

    let made = [
        convert(&["tests/programs/marie-add.asm", &mas, "--machine", "marie"]),
        convert(&[&mas, &json]),
    ];
    let runs = [&mas, &json].map(|path| run(&[path, "--input", "12,30"]).0);

    for path in [&mas, &json] {
        fs::remove_file(path).unwrap();
    }

    assert_eq!(made.map(|(_, _, code)| code), [0, 0]);
    assert_eq!(runs, ["42\n", "42\n"]);
}

#[test]
fn convert_errors_exit_with_1() {
    let out = temp("refused.ram");

    for (args, message) in [
        (&["assets/countdown.json", &out][..], "radix 10"),
        (&[ADD, &out], "--machine"),
        (&["assets/sum.ram"], "a file is missing"),
        (&["assets/sum.ram", &out, &out], "one file too many"),
        (&["assets/sum.ram", &out, "--trace"], "unknown option --trace"),
        (&["tests/programs/missing.ram", &out], "missing.ram"),
    ] {
        let (stdout, stderr, code) = convert(args);

        assert_eq!((stdout.as_str(), code), ("", 1), "{args:?}");
        assert!(stderr.starts_with("error: "), "{args:?}: {stderr}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
    }

    assert!(fs::metadata(&out).is_err());
}

#[test]
fn the_step_limit_exits_with_2() {
    let (stdout, stderr, code) = run(&[
        "tests/programs/lmc-countdown.asm",
        "--machine",
        "lmc",
        "--input",
        "5",
        "--max-steps",
        "6",
    ]);

    assert_eq!((stdout.as_str(), code), ("5\n4\n", 2));
    assert!(stderr.contains("step limit of 6"), "{stderr}");
}

#[test]
fn running_out_of_input_exits_with_3() {
    let (stdout, stderr, code) = run(&[ADD, "--machine", "lmc", "--input", "12"]);

    assert_eq!((stdout.as_str(), code), ("", 3));
    assert!(stderr.contains("waiting for input"), "{stderr}");
}

#[test]
fn machine_errors_go_to_stderr() {
    let path = temp("divide-by-zero.asm");
    fs::write(&path, "IN\nOUT\nDIV 0\n").unwrap();

    let (stdout, stderr, code) = run(&[&path, "--machine", "standard", "--input", "7"]);
    fs::remove_file(&path).unwrap();

    assert_eq!((stdout.as_str(), code), ("7\n", 0));
    assert!(stderr.contains("error: division by zero"), "{stderr}");
}

#[test]
fn the_trace_shows_an_instruction_before_its_error() {
    let path = temp("divide-by-zero-traced.asm");
    fs::write(&path, "DIV 0\n").unwrap();

    let (_, stderr, _) = run(&[&path, "--machine", "standard", "--trace"]);
    fs::remove_file(&path).unwrap();

    assert!(
        stderr.starts_with("00  06.00  DIV 00  acc 0\nerror: division by zero\n"),
        "{stderr}"
    );
}

#[test]
fn a_mas_file_runs_only_as_marie() {
    let (mas, out) = (temp("echo.mas"), temp("echo.json"));
    fs::write(&mas, "Input\nOutput\nHalt\n").unwrap();

    let refused = [
        run(&[&mas, "--machine", "lmc", "--input", "5"]),
        convert(&[&mas, &out, "--machine", "lmc"]),
    ];
    let marie = run(&[&mas, "--machine", "MARIE", "--input", "5"]);
    fs::remove_file(&mas).unwrap();

    for (stdout, stderr, code) in refused {
        assert_eq!((stdout.as_str(), code), ("", 1));
        assert!(stderr.contains("always MARIE"), "{stderr}");
    }

    assert!(fs::metadata(&out).is_err());
    assert_eq!((marie.0.as_str(), marie.2), ("5\n", 0));
}

#[test]
fn help_is_printed() {
    let (stdout, _, code) = run(&["--help"]);

    assert_eq!(code, 0);
    assert!(stdout.contains("--max-steps"));
}

#[test]
fn cardiac_example_prints_the_digits() {
    let (stdout, _, code) = run(&["assets/cardiac-digits.asm", "--machine", "cardiac"]);

    assert_eq!((stdout.as_str(), code), ("4\n7\n2\n", 0));
}
