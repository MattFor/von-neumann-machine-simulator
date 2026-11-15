use std::process::Command;

const ADD: &str = "tests/programs/lmc-add.asm";

// stdout, stderr and the exit code of the binary's `run`.
fn run(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_von-neumann-machine-simulator"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("run")
        .args(args)
        .output()
        .unwrap();

    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.code().unwrap(),
    )
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
    let path = std::env::temp_dir().join("vnm-cli-divide-by-zero.asm");
    std::fs::write(&path, "IN\nOUT\nDIV 0\n").unwrap();

    let path = path.to_str().unwrap();
    let (stdout, stderr, code) = run(&[path, "--machine", "standard", "--input", "7"]);
    std::fs::remove_file(path).unwrap();

    assert_eq!((stdout.as_str(), code), ("7\n", 0));
    assert!(stderr.contains("error: division by zero"), "{stderr}");
}

#[test]
fn help_is_printed() {
    let (stdout, _, code) = run(&["--help"]);

    assert_eq!(code, 0);
    assert!(stdout.contains("--max-steps"));
}
