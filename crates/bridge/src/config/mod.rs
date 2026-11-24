//! Runtime settings, read once from the environment and validated, so a typo fails at start-up
//! instead of mid-game.

use std::env::{self, VarError};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use dsu::Port;
use openvr_sys::Hand;

const PORT_VAR: &str = "BRIDGE_PORT";
const HAND_VAR: &str = "BRIDGE_HAND";
/// Diagnostic: sweep known pointer values, ignoring the controller.
const POINTER_TEST_VAR: &str = "PTR_TEST";
/// File to stream controller movement, pointer and sensors to (JSON lines).
const TELEMETRY_VAR: &str = "BRIDGE_TELEMETRY";

/// Where the pointer comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerSource {
    /// Where the controller aims.
    Controller,
    /// The `PTR_TEST` diagnostic: known values in turn, ignoring the controller.
    Sweep,
}

/// The validated settings of one bridge instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub port: Port,
    /// The controller this bridge reads; `Left` is the second instance (Wii Remote 2).
    pub hand: Hand,
    pub pointer_source: PointerSource,
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
    /// This bridge is the second instance: the left controller is Wii Remote 2.
    #[must_use]
    pub fn is_player2(&self) -> bool {
        self.hand == Hand::Left
    }

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
        let port = read(PORT_VAR)?.map_or(Ok(Port::DEFAULT), |raw| parse_port(&raw))?;
        let hand = read(HAND_VAR)?.map_or(Ok(Hand::Right), |raw| parse_hand(&raw))?;
        // Like the TypeScript `Boolean(...)` / truthiness checks: set but empty counts as unset.
        let is_pointer_test = read(POINTER_TEST_VAR)?.is_some_and(|raw| !raw.is_empty());
        let pointer_source = if is_pointer_test {
            PointerSource::Sweep
        } else {
            PointerSource::Controller
        };
        let telemetry_path = read(TELEMETRY_VAR)?
            .filter(|raw| !raw.is_empty())
            .map(PathBuf::from);
        Ok(Self {
            port,
            hand,
            pointer_source,
            telemetry_path,
        })
    }
}

/// A whole number 1..65535, written as `Number()` reads it (surrounding blanks ignored).
fn parse_port(raw: &str) -> Result<Port, ConfigError> {
    let invalid = || ConfigError {
        variable: PORT_VAR,
        expected: "an integer from 1 to 65535",
        received: raw.to_owned(),
    };
    let value: f64 = raw.trim().parse().map_err(|_err| invalid())?;
    let is_whole = value.fract() == 0.0;
    if !is_whole {
        return Err(invalid());
    }
    numeric::truncate_to::<u16>(value)
        .and_then(Port::new)
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
mod tests;
