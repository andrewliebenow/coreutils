// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// spell-checker:ignore parenb parmrk ixany iuclc onlcr ofdel icanon noflsh ixon

use crate::common::util::{TestScenario, UCommand};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{fs::File, io::Read, process::Stdio};

const DEV_TTY: &str = "/dev/tty";

fn get_print_first_line_regex() -> &'static Regex {
    static ONCE_CELL: OnceCell<Regex> = OnceCell::<Regex>::new();

    ONCE_CELL.get_or_init(|| {
        // e.g.:
        // speed 38400 baud; line = 0;
        Regex::new("speed [0-9]+ baud; line = [0-9]+;").unwrap()
    })
}

fn get_print_dash_a_first_line_regex() -> &'static Regex {
    static ONCE_CELL: OnceCell<Regex> = OnceCell::<Regex>::new();

    ONCE_CELL.get_or_init(|| {
        // e.g.:
        // speed 38400 baud; rows 54; columns 216; line = 0;
        Regex::new("speed [0-9]+ baud; rows [0-9]+; columns [0-9]+; line = [0-9]+;").unwrap()
    })
}

impl UCommand {
    fn set_stdin_to_dev_tty_stdio(&mut self) -> &mut Self {
        let file = File::open(DEV_TTY).unwrap();

        let stdio = Stdio::from(file);

        self.set_stdin(stdio)
    }

    fn conditional_set_stdin_to_dev_tty_stdio(&mut self, set_stdin: bool) -> &mut Self {
        if set_stdin {
            self.set_stdin_to_dev_tty_stdio()
        } else {
            self
        }
    }
}

#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn test_invalid_arg() {
    new_ucmd!()
        .arg("--definitely-invalid")
        .set_stdin_to_dev_tty_stdio()
        .fails()
        .code_is(1);
}

#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn runs() {
    new_ucmd!().set_stdin_to_dev_tty_stdio().succeeds();
}

#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn print_all() {
    let cmd_result = new_ucmd!()
        .arg("-a")
        .set_stdin_to_dev_tty_stdio()
        .succeeds();

    // "iuclc" removed due to this comment in stty.rs:
    //
    // not supported by nix
    // Flag::new("iuclc", I::IUCLC),

    // Random selection of flags to check for
    for flag in [
        "parenb", "parmrk", "ixany", "onlcr", "ofdel", "icanon", "noflsh",
    ] {
        cmd_result.stdout_contains(flag);
    }
}

#[test]
fn save_and_setting() {
    new_ucmd!()
        .args(&["--save", "nl0"])
        .fails()
        .stderr_contains("when specifying an output style, modes may not be set");
}

#[test]
fn all_and_setting() {
    new_ucmd!()
        .args(&["--all", "nl0"])
        .fails()
        .stderr_contains("when specifying an output style, modes may not be set");
}

#[test]
fn save_and_all() {
    new_ucmd!()
        .args(&["--save", "--all"])
        .fails()
        .stderr_contains(
            "the options for verbose and stty-readable output styles are mutually exclusive",
        );

    new_ucmd!()
        .args(&["--all", "--save"])
        .fails()
        .stderr_contains(
            "the options for verbose and stty-readable output styles are mutually exclusive",
        );
}

// Make sure the "allow_hyphen_values" clap function has been called with true
#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn negation() {
    new_ucmd!()
        .arg("-ixon")
        .set_stdin_to_dev_tty_stdio()
        .succeeds()
        .stdout_is_bytes([])
        .stderr_is_bytes([]);
}

fn run_and_check_print_should_succeed(args: &[&str], stdout_regex: &Regex, set_stdin: bool) {
    new_ucmd!()
        .args(args)
        .conditional_set_stdin_to_dev_tty_stdio(set_stdin)
        .succeeds()
        .stdout_str_check(|st| {
            let Some(str) = st.lines().next() else {
                return false;
            };

            stdout_regex.is_match(str)
        })
        .no_stderr();
}

// The end of options delimiter ("--") and everything after must be ignored
#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn ignore_end_of_options_and_after() {
    {
        // "stty -a -- -ixon" should behave like "stty -a"
        // Should not abort with an error complaining about passing both "-a" and "-ixon" (since "-ixon" is after "--")
        run_and_check_print_should_succeed(
            &["-a", "--", "-ixon"],
            get_print_dash_a_first_line_regex(),
            true,
        );
    }

    {
        // "stty -- non-existent-option-that-must-be-ignore" should behave like "stty"
        // Should not abort with an error complaining about an invalid argument, since the invalid argument is after "--"
        run_and_check_print_should_succeed(
            &["--", "non-existent-option-that-must-be-ignored"],
            get_print_first_line_regex(),
            true,
        );
    }
}

#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn f_file_option() {
    for st in ["-F", "--file"] {
        for bo in [false, true] {
            let ar: [&str; 3];
            let arr: [&str; 2];

            let (args, regex): (&[&str], &'static Regex) = if bo {
                ar = [st, DEV_TTY, "-a"];

                (&ar, get_print_dash_a_first_line_regex())
            } else {
                arr = [st, DEV_TTY];

                (&arr, get_print_first_line_regex())
            };

            run_and_check_print_should_succeed(args, regex, false);
        }
    }
}

// Make sure stty is using stdin to look up terminal attributes, not stdout
#[test]
#[cfg(not(target_os = "android"))]
#[ignore = "These tests should work locally, but /dev/tty isn't configured when running in CI"]
fn correct_file_descriptor_output_piped() {
    const PIPE_STDOUT_TO: &str = "pipe_stdout_to";
    const PIPE_STDERR_TO: &str = "pipe_stderr_to";

    let test_scenario = TestScenario::new(util_name!());

    let at_path = &test_scenario.fixtures;

    let stdout_file = at_path.make_file(PIPE_STDOUT_TO);
    let stderr_file = at_path.make_file(PIPE_STDERR_TO);

    test_scenario
        .ucmd()
        .set_stdin_to_dev_tty_stdio()
        .set_stdout(Stdio::from(stdout_file))
        .set_stderr(Stdio::from(stderr_file))
        .succeeds();

    let mut read_to_string_buffer = String::new();

    at_path
        .open(PIPE_STDOUT_TO)
        .read_to_string(&mut read_to_string_buffer)
        .unwrap();

    let stdout_first_line = read_to_string_buffer.lines().next().unwrap();

    assert!(get_print_first_line_regex().is_match(stdout_first_line));

    read_to_string_buffer.clear();

    at_path
        .open(PIPE_STDERR_TO)
        .read_to_string(&mut read_to_string_buffer)
        .unwrap();

    assert!(read_to_string_buffer.is_empty());
}
