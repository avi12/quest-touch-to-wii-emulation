//! Runtime settings, read once from the environment and validated, so a typo fails at start-up
//! instead of mid-game.

use std::env::{self, VarError};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use openvr_sys::Hand;

const PORT_VAR: &str = "BRIDGE_PORT";
const HAND_VAR: &str = "BRIDGE_HAND";
/// Diagnostic: sweep known pointer values, ignoring the controller.
const POINTER_TEST_VAR: &str = "PTR_TEST";
/// File to stream controller movement, pointer and sensors to (JSON lines).
const TELEMETRY_VAR: &str = "BRIDGE_TELEMETRY";
const DEFAULT_PORT: u16 = 26760;
const MIN_PORT: u16 = 1;

/// The validated settings of one bridge instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub port: u16,
    /// The controller this bridge reads; `Left` is the second instance (Wii Remote 2).
    pub hand: Hand,
    pub is_player2: bool,
    pub is_pointer_test: bool,
    pub telemetry_path: Option<PathBuf>,
}

/// An environment variable that does not hold an accepted value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub variable: &'static str,
    pub expected: &'static str,
    pub received: String,
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} must be {}, got {:?}",
            self.variable, self.expected, self.received
        )
    }
}

impl Error for ConfigError {}

/// The name `BRIDGE_HAND` uses for `hand`.
pub fn hand_name(hand: Hand) -> &'static str {
    match hand {
        Hand::Left => "left",
        Hand::Right => "right",
    }
}

impl Config {
    /// Reads the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| env::var(name))
    }

    /// Reads the settings through `lookup` (`std::env::var` or a test table).
    pub fn from_lookup(
        lookup: impl Fn(&str) -> Result<String, VarError>,
    ) -> Result<Self, ConfigError> {
        let read = |variable: &'static str| match lookup(variable) {
            Ok(value) => Ok(Some(value)),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(raw)) => Err(ConfigError {
                variable,
                expected: "valid Unicode",
                received: raw.to_string_lossy().into_owned(),
            }),
        };
        let port = read(PORT_VAR)?.map_or(Ok(DEFAULT_PORT), |raw| parse_port(&raw))?;
        let hand = read(HAND_VAR)?.map_or(Ok(Hand::Right), |raw| parse_hand(&raw))?;
        // Like the TypeScript `Boolean(...)` / truthiness checks: set but empty counts as unset.
        let is_pointer_test = read(POINTER_TEST_VAR)?.is_some_and(|raw| !raw.is_empty());
        let telemetry_path = read(TELEMETRY_VAR)?
            .filter(|raw| !raw.is_empty())
            .map(PathBuf::from);
        Ok(Self {
            port,
            hand,
            is_player2: hand == Hand::Left,
            is_pointer_test,
            telemetry_path,
        })
    }
}

/// A whole number 1..65535, written as `Number()` reads it (surrounding blanks ignored).
fn parse_port(raw: &str) -> Result<u16, ConfigError> {
    let invalid = || ConfigError {
        variable: PORT_VAR,
        expected: "an integer from 1 to 65535",
        received: raw.to_owned(),
    };
    let value: f64 = raw.trim().parse().map_err(|_err| invalid())?;
    if value.fract() != 0.0 {
        return Err(invalid());
    }
    num_traits::ToPrimitive::to_u16(&value)
        .filter(|port| *port >= MIN_PORT)
        .ok_or_else(invalid)
}

fn parse_hand(raw: &str) -> Result<Hand, ConfigError> {
    [Hand::Left, Hand::Right]
        .into_iter()
        .find(|hand| hand_name(*hand) == raw)
        .ok_or_else(|| ConfigError {
            variable: HAND_VAR,
            expected: "\"left\" or \"right\"",
            received: raw.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn parse(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let table: HashMap<String, String> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        Config::from_lookup(|name| table.get(name).cloned().ok_or(VarError::NotPresent))
    }

    #[test]
    fn defaults_to_the_right_controller_on_26760() {
        let config = parse(&[]).unwrap();
        assert_eq!(config.port, 26760);
        assert_eq!(config.hand, Hand::Right);
        assert!(!config.is_player2 && !config.is_pointer_test);
        assert_eq!(config.telemetry_path, None);
    }

    #[test]
    fn reads_the_second_instance() {
        let config = parse(&[
            ("BRIDGE_PORT", " 26761 "),
            ("BRIDGE_HAND", "left"),
            ("PTR_TEST", "1"),
            ("BRIDGE_TELEMETRY", "t.jsonl"),
        ])
        .unwrap();
        assert_eq!((config.port, config.hand), (26761, Hand::Left));
        assert!(config.is_player2 && config.is_pointer_test);
        assert_eq!(config.telemetry_path, Some(PathBuf::from("t.jsonl")));
    }

    #[test]
    fn empty_flags_count_as_unset() {
        let config = parse(&[("PTR_TEST", ""), ("BRIDGE_TELEMETRY", "")]).unwrap();
        assert!(!config.is_pointer_test);
        assert_eq!(config.telemetry_path, None);
    }

    #[test]
    fn rejects_what_zod_rejects() {
        for port in ["0", "65536", "1.5", "abc", "", "-1"] {
            let error = parse(&[("BRIDGE_PORT", port)]).unwrap_err();
            assert_eq!(
                (error.variable, error.received.as_str()),
                ("BRIDGE_PORT", port)
            );
        }
        for hand in ["Left", "", "both"] {
            assert_eq!(
                parse(&[("BRIDGE_HAND", hand)]).unwrap_err().variable,
                "BRIDGE_HAND"
            );
        }
        assert_eq!(parse(&[("BRIDGE_PORT", "1e3")]).unwrap().port, 1000);
    }

    #[test]
    fn error_names_the_variable_and_value() {
        let error = parse(&[("BRIDGE_PORT", "abc")]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "BRIDGE_PORT must be an integer from 1 to 65535, got \"abc\""
        );
    }
}
