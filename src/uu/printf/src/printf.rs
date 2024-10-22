// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#![allow(dead_code)]

use clap::builder::ValueParser;
use clap::{crate_version, Arg, ArgAction, Command};
use std::ffi::OsString;
use std::io::stdout;
use std::ops::ControlFlow;
use uucore::error::{UResult, UUsageError};
use uucore::format::{bytes_from_os_str, parse_spec_and_escape, FormatArgument, FormatItem};
use uucore::{format_usage, help_about, help_section, help_usage};

const VERSION: &str = "version";
const HELP: &str = "help";
const USAGE: &str = help_usage!("printf.md");
const ABOUT: &str = help_about!("printf.md");
const AFTER_HELP: &str = help_section!("after help", "printf.md");

mod options {
    pub const FORMAT: &str = "FORMAT";
    pub const ARGUMENT: &str = "ARGUMENT";
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().get_matches_from(args);

    let format = matches
        .get_one::<OsString>(options::FORMAT)
        .ok_or_else(|| UUsageError::new(1, "missing operand"))?;

    let format_bytes = bytes_from_os_str(format)?;

    let values = match matches.get_many::<OsString>(options::ARGUMENT) {
        Some(os_string) => os_string
            .map(|os_string_ref| FormatArgument::Unparsed(os_string_ref.to_owned()))
            .collect(),
        None => Vec::<FormatArgument>::new(),
    };

    let mut format_seen = false;
    let mut args = values.iter().peekable();

    for item in parse_spec_and_escape(format_bytes) {
        if let Ok(FormatItem::Spec(_)) = item {
            format_seen = true;
        }
        match item?.write(stdout(), &mut args)? {
            ControlFlow::Continue(()) => {}
            ControlFlow::Break(()) => return Ok(()),
        };
    }

    // Without format specs in the string, the iter would not consume any args,
    // leading to an infinite loop. Thus, we exit early.
    if !format_seen {
        return Ok(());
    }

    while args.peek().is_some() {
        for item in parse_spec_and_escape(format_bytes) {
            match item?.write(stdout(), &mut args)? {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(()) => return Ok(()),
            };
        }
    }
    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .allow_hyphen_values(true)
        .version(crate_version!())
        .about(ABOUT)
        .after_help(AFTER_HELP)
        .override_usage(format_usage(USAGE))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new(HELP)
                .long(HELP)
                .help("Print help information")
                .action(ArgAction::Help),
        )
        .arg(
            Arg::new(VERSION)
                .long(VERSION)
                .help("Print version information")
                .action(ArgAction::Version),
        )
        .arg(Arg::new(options::FORMAT).value_parser(ValueParser::os_string()))
        .arg(
            Arg::new(options::ARGUMENT)
                .action(ArgAction::Append)
                .value_parser(ValueParser::os_string()),
        )
}
