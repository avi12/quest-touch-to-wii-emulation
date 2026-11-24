//! A command-line argument that failed validation, and how it is reported.

use std::fmt;
use std::process::ExitCode;

/// Exit code for a bad command line.
pub const EXIT_BAD_USAGE: u8 = 2;

/// A command-line argument that failed validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgError {
    pub message: String,
}

impl ArgError {
    pub(crate) fn new(name: &str, expected: &str, raw: &str) -> Self {
        Self {
            message: format!("{name}: expected {expected}, got \"{raw}\""),
        }
    }

    /// Prints the error after a usage line and returns the bad-usage exit code.
    pub fn report(&self, usage: &str) -> ExitCode {
        eprintln!("usage: {usage}\n{self}");
        ExitCode::from(EXIT_BAD_USAGE)
    }
}

impl fmt::Display for ArgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ArgError {}
