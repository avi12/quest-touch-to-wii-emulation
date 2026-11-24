//! `telemetry_tail [file] [seconds] [--rows] [--marks]`.

use std::path::PathBuf;
use std::time::Duration;

use crate::cli::{ArgError, has_flag, positional, positive_duration};

const DEFAULT_FILE: &str = "telemetry-right.jsonl";
const DEFAULT_WINDOW: Duration = Duration::from_secs(10);

/// The usage line shown with an argument error.
pub const USAGE: &str = "telemetry_tail [file] [seconds > 0] [--rows] [--marks]";

/// Which stream to read and what to print.
#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryOptions {
    pub file: PathBuf,
    /// How far back from the newest sample to summarise (ignored with `--marks`).
    pub window: Duration,
    /// `--rows`: also print one row per 250 ms.
    pub has_rows: bool,
    /// `--marks`: analyse each stretch of B held as one recording.
    pub has_marks: bool,
}

impl TelemetryOptions {
    pub fn parse(args: &[String]) -> Result<Self, ArgError> {
        let values = positional(args);
        Ok(Self {
            file: PathBuf::from(values.first().map_or(DEFAULT_FILE, |file| file)),
            window: positive_duration(values.get(1).copied(), "seconds", DEFAULT_WINDOW)?,
            has_rows: has_flag(args, "--rows"),
            has_marks: has_flag(args, "--marks"),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use crate::telemetry::TelemetryOptions;
    use crate::telemetry::options::DEFAULT_FILE;

    fn parse(args: &[&str]) -> Option<TelemetryOptions> {
        let owned: Vec<String> = args.iter().map(ToString::to_string).collect();
        TelemetryOptions::parse(&owned).ok()
    }

    #[test]
    fn parses_positionals_and_flags() {
        let options = parse(&["--marks", "left.jsonl", "5"]);
        assert_eq!(
            options.as_ref().map(|parsed| parsed.file.as_path()),
            Some(Path::new("left.jsonl"))
        );
        assert_eq!(
            options.as_ref().map(|parsed| parsed.window),
            Some(Duration::from_secs(5))
        );
        assert!(options.is_some_and(|parsed| parsed.has_marks && !parsed.has_rows));
        assert_eq!(
            parse(&[]).map(|parsed| parsed.file),
            Some(Path::new(DEFAULT_FILE).to_owned())
        );
        assert!(parse(&["x", "-1"]).is_none());
    }
}
