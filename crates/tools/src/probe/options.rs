//! `dsu_probe [port] [seconds]`.

use crate::args::{ArgError, port, positional, positive_number};

const DEFAULT_PORT: u16 = 26760;
const DEFAULT_SECONDS: f64 = 5.0;

/// The usage line shown with an argument error.
pub const USAGE: &str = "dsu_probe [port 1-65535] [seconds > 0]";

/// What to probe and for how long.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbeOptions {
    pub port: u16,
    pub seconds: f64,
}

impl ProbeOptions {
    pub fn parse(args: &[String]) -> Result<Self, ArgError> {
        let values = positional(args);
        Ok(Self {
            port: port(values.first().copied(), "port", DEFAULT_PORT)?,
            seconds: positive_number(values.get(1).copied(), "seconds", DEFAULT_SECONDS)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_overrides() {
        let defaults = ProbeOptions::parse(&[]).unwrap();
        assert_eq!(defaults.port, DEFAULT_PORT);
        let given = ProbeOptions::parse(&["26761".to_owned(), "2".to_owned()]).unwrap();
        assert_eq!(
            given,
            ProbeOptions {
                port: 26761,
                seconds: 2.0
            }
        );
        assert!(ProbeOptions::parse(&["0".to_owned()]).is_err());
    }
}
