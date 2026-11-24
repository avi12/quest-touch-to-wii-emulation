//! Positional arguments, `--flags` and durations given in seconds.

use std::time::Duration;

use crate::cli::ArgError;

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

/// A finite number of seconds above zero, or `default` when the argument is absent.
pub fn positive_duration(
    raw: Option<&str>,
    name: &str,
    default: Duration,
) -> Result<Duration, ArgError> {
    let Some(text) = raw else {
        return Ok(default);
    };
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
        .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| ArgError::new(name, "a number > 0", text))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::cli::{has_flag, positional, positive_duration};

    const DEFAULT: Duration = Duration::from_secs(5);

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
    fn validates_seconds() {
        assert_eq!(positive_duration(None, "seconds", DEFAULT), Ok(DEFAULT));
        assert_eq!(
            positive_duration(Some("2.5"), "seconds", DEFAULT),
            Ok(Duration::from_millis(2500))
        );
        assert!(positive_duration(Some("0"), "seconds", DEFAULT).is_err());
        assert!(positive_duration(Some("abc"), "seconds", DEFAULT).is_err());
        assert!(positive_duration(Some("inf"), "seconds", DEFAULT).is_err());
    }
}
