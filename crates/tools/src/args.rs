//! Small typed command-line parsing: positional arguments, `--flags`, validated numbers.

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
    fn new(name: &str, expected: &str, raw: &str) -> Self {
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

/// The command-line arguments after the program name.
pub fn command_line() -> Vec<String> {
    std::env::args().skip(1).collect()
}

/// Arguments that are not `--flags`, in order.
pub fn positional(args: &[String]) -> Vec<&str> {
    args.iter()
        .map(String::as_str)
        .filter(|arg| !arg.starts_with("--"))
        .collect()
}

/// Whether `flag` (for example `--rows`) was given.
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// A finite number above zero, or `default` when the argument is absent.
pub fn positive_number(raw: Option<&str>, name: &str, default: f64) -> Result<f64, ArgError> {
    let Some(text) = raw else {
        return Ok(default);
    };
    match text.trim().parse::<f64>() {
        Ok(value) if value.is_finite() && value > 0.0 => Ok(value),
        _ => Err(ArgError::new(name, "a number > 0", text)),
    }
}

/// A UDP port 1-65535, or `default` when the argument is absent.
pub fn port(raw: Option<&str>, name: &str, default: u16) -> Result<u16, ArgError> {
    let Some(text) = raw else {
        return Ok(default);
    };
    match text.trim().parse::<u16>() {
        Ok(value) if value >= 1 => Ok(value),
        _ => Err(ArgError::new(name, "an integer 1-65535", text)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn splits_positional_from_flags() {
        let args = strings(&["file.jsonl", "--marks", "5"]);
        assert_eq!(positional(&args), ["file.jsonl", "5"]);
        assert!(has_flag(&args, "--marks"));
        assert!(!has_flag(&args, "--rows"));
    }

    #[test]
    fn validates_numbers() {
        assert_eq!(positive_number(None, "seconds", 5.0), Ok(5.0));
        assert_eq!(positive_number(Some("2.5"), "seconds", 5.0), Ok(2.5));
        assert!(positive_number(Some("0"), "seconds", 5.0).is_err());
        assert!(positive_number(Some("abc"), "seconds", 5.0).is_err());
        assert!(positive_number(Some("inf"), "seconds", 5.0).is_err());
        assert_eq!(port(Some("26761"), "port", 1), Ok(26761));
        assert!(port(Some("0"), "port", 1).is_err());
        assert!(port(Some("70000"), "port", 1).is_err());
    }
}
