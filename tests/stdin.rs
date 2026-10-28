//! `scanf!` reads the process stdin, so each case runs in a child process (this same binary
//! started again) with piped input. There is no test harness: `main` is both the runner and,
//! when `CASE_ENV` is set, the child.

use scanf::{fscanf, scanf};
use std::io::{ErrorKind, Write};
use std::process::{Command, Stdio};

const CASE_ENV: &str = "SCANF_STDIN_CASE";

/// `Ok("value")` or `Err(Kind)`, so the parent can compare whole outcomes as text.
fn describe<T: std::fmt::Debug>(result: std::io::Result<T>) -> String {
    match result {
        Ok(value) => format!("Ok({value:?})"),
        Err(error) => format!("Err({:?})", error.kind()),
    }
}

/// What the child prints for each case.
fn run_case(case: &str) -> String {
    match case {
        "return_mode" => {
            let result: std::io::Result<String> = scanf!("{}");
            describe(result)
        }
        "two_lines" => {
            let first: std::io::Result<String> = scanf!("{}");
            let second: std::io::Result<String> = scanf!("{}");
            format!("{} {}", describe(first), describe(second))
        }
        "named" => {
            let mut number: i32 = 0;
            let result = scanf!("x={number}");
            format!("{} {number}", describe(result))
        }
        "named_with_trailing_text" => {
            let mut number: i32 = 0;
            let result = scanf!("x={number} end");
            format!("{} {number}", describe(result))
        }
        "explicit_argument" => {
            let mut number: i32 = 0;
            let result = scanf!("{}", &mut number);
            format!("{} {number}", describe(result))
        }
        "user_buffer" => {
            let buffer = String::from("mine");
            let text: String = scanf!("{}").unwrap_or_default();
            format!("{buffer} {text}")
        }
        "pair" => {
            let result: std::io::Result<(i32, String)> = scanf!("{},{}");
            describe(result)
        }
        "fscanf_on_a_held_lock" => {
            // `scanf!` here would wait forever for the lock this thread already holds.
            let mut input = std::io::stdin().lock();
            let first: std::io::Result<String> = fscanf!(input, "{}");
            let second: std::io::Result<(i32, i32)> = fscanf!(input, "{},{}");
            let third: std::io::Result<String> = fscanf!(input, "{}");
            format!(
                "{} {} {}",
                describe(first),
                describe(second),
                describe(third)
            )
        }
        "fscanf_on_stdin_handle" => {
            let stdin = std::io::stdin();
            let n: std::io::Result<i32> = fscanf!(stdin, "{}");
            describe(n)
        }
        other => panic!("unknown case {other:?}"),
    }
}

struct Case {
    name: &'static str,
    case: &'static str,
    stdin: &'static [u8],
    expected: &'static str,
}

const CASES: &[Case] = &[
    Case {
        name: "line_ending_lf",
        case: "return_mode",
        stdin: b"hello\n",
        expected: "Ok(\"hello\")",
    },
    Case {
        name: "line_ending_crlf",
        case: "return_mode",
        stdin: b"hello\r\n",
        expected: "Ok(\"hello\")",
    },
    Case {
        name: "no_trailing_newline",
        case: "return_mode",
        stdin: b"hello",
        expected: "Ok(\"hello\")",
    },
    Case {
        name: "empty_line_is_an_empty_value",
        case: "return_mode",
        stdin: b"\n",
        expected: "Ok(\"\")",
    },
    Case {
        name: "only_one_line_ending_is_removed",
        case: "return_mode",
        stdin: b"a\r\r\n",
        expected: "Ok(\"a\\r\")",
    },
    Case {
        name: "lone_cr_is_a_line_ending",
        case: "return_mode",
        stdin: b"a\r",
        expected: "Ok(\"a\")",
    },
    Case {
        name: "empty_stdin_is_unexpected_eof",
        case: "return_mode",
        stdin: b"",
        expected: "Err(UnexpectedEof)",
    },
    Case {
        name: "each_call_consumes_one_line",
        case: "two_lines",
        stdin: b"one\ntwo\n",
        expected: "Ok(\"one\") Ok(\"two\")",
    },
    Case {
        name: "second_call_after_the_last_line_is_unexpected_eof",
        case: "two_lines",
        stdin: b"one\n",
        expected: "Ok(\"one\") Err(UnexpectedEof)",
    },
    Case {
        name: "named_placeholder",
        case: "named",
        stdin: b"x=42\r\n",
        expected: "Ok(()) 42",
    },
    Case {
        name: "named_placeholder_on_empty_stdin_leaves_the_variable_alone",
        case: "named",
        stdin: b"",
        expected: "Err(UnexpectedEof) 0",
    },
    #[cfg(not(feature = "ignore-trailing-input"))]
    Case {
        name: "text_after_the_last_literal_is_an_error_and_writes_nothing",
        case: "named_with_trailing_text",
        stdin: b"x=42 end extra\n",
        expected: "Err(InvalidInput) 0",
    },
    #[cfg(feature = "ignore-trailing-input")]
    Case {
        name: "text_after_the_last_literal_is_ignored",
        case: "named_with_trailing_text",
        stdin: b"x=42 end extra\n",
        expected: "Ok(()) 42",
    },
    Case {
        name: "trailing_text_that_matches_is_accepted",
        case: "named_with_trailing_text",
        stdin: b"x=42 end\n",
        expected: "Ok(()) 42",
    },
    Case {
        name: "fscanf_reads_from_a_lock_that_is_already_held",
        case: "fscanf_on_a_held_lock",
        stdin: b"one\n2,3\n",
        expected: "Ok(\"one\") Ok((2, 3)) Err(UnexpectedEof)",
    },
    Case {
        name: "fscanf_accepts_the_stdin_handle",
        case: "fscanf_on_stdin_handle",
        stdin: b"12\n",
        expected: "Ok(12)",
    },
    Case {
        name: "explicit_argument",
        case: "explicit_argument",
        stdin: b"17\n",
        expected: "Ok(()) 17",
    },
    Case {
        name: "number_ending_in_a_lone_cr_parses",
        case: "explicit_argument",
        stdin: b"42\r",
        expected: "Ok(()) 42",
    },
    Case {
        name: "user_variable_named_buffer_is_untouched",
        case: "user_buffer",
        stdin: b"text\n",
        expected: "mine text",
    },
    Case {
        name: "tuple_in_return_mode",
        case: "pair",
        stdin: b"3,abc\r\n",
        expected: "Ok((3, \"abc\"))",
    },
];

fn run_in_child(case: &Case) -> Result<(), String> {
    let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .env(CASE_ENV, case.case)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // The child may exit without reading everything, so a broken pipe is not an error here.
    let mut stdin = child.stdin.take().expect("stdin was piped");
    let write_result = stdin.write_all(case.stdin);
    drop(stdin);
    if let Err(error) = write_result
        && error.kind() != ErrorKind::BrokenPipe
    {
        return Err(error.to_string());
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!(
            "child failed: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if stdout.trim_end() != case.expected {
        return Err(format!(
            "expected {:?}, got {:?}",
            case.expected,
            stdout.trim_end()
        ));
    }
    Ok(())
}

fn main() {
    if let Ok(case) = std::env::var(CASE_ENV) {
        println!("{}", run_case(&case));
        return;
    }

    let mut failures = 0;
    println!("running {} tests", CASES.len());
    for case in CASES {
        match run_in_child(case) {
            Ok(()) => println!("test {} ... ok", case.name),
            Err(message) => {
                failures += 1;
                println!("test {} ... FAILED\n    {message}", case.name);
            }
        }
    }
    println!(
        "\ntest result: {}. {} passed; {failures} failed",
        if failures == 0 { "ok" } else { "FAILED" },
        CASES.len() - failures
    );
    if failures > 0 {
        std::process::exit(1);
    }
}
