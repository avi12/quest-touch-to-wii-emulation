//! `telemetry_tail [file] [seconds] [--rows] [--marks]`.

use crate::args::{ArgError, has_flag, positional, positive_number};

const DEFAULT_FILE: &str = "telemetry-right.jsonl";
const DEFAULT_SECONDS: f64 = 10.0;

/// The usage line shown with an argument error.
pub const USAGE: &str = "telemetry_tail [file] [seconds > 0] [--rows] [--marks]";

/// Which stream to read and what to print.
#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryOptions {
    pub file: String,
    /// How far back from the newest sample to summarise (ignored with `--marks`).
    pub seconds: f64,
    /// Also print one row per 250 ms.
    pub rows: bool,
    /// Analyse each stretch of B held as one recording.
    pub marks: bool,
}

impl TelemetryOptions {
    pub fn parse(args: &[String]) -> Result<Self, ArgError> {
        let values = positional(args);
        Ok(Self {
            file: values.first().map_or(DEFAULT_FILE, |file| file).to_owned(),
            seconds: positive_number(values.get(1).copied(), "seconds", DEFAULT_SECONDS)?,
            rows: has_flag(args, "--rows"),
            marks: has_flag(args, "--marks"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_positionals_and_flags() {
        let args: Vec<String> = ["--marks", "left.jsonl", "5"].map(String::from).to_vec();
        let options = TelemetryOptions::parse(&args).unwrap();
        assert_eq!(options.file, "left.jsonl");
        assert!((options.seconds - 5.0).abs() < f64::EPSILON);
        assert!(options.marks && !options.rows);
        assert_eq!(TelemetryOptions::parse(&[]).unwrap().file, DEFAULT_FILE);
        assert!(TelemetryOptions::parse(&["x".to_owned(), "-1".to_owned()]).is_err());
    }
}
