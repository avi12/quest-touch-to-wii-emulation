//! `dsu_probe [port] [seconds]`.

use std::time::Duration;

use dsu::Port;

use crate::cli::{ArgError, parse_port, positional, positive_duration};

const DEFAULT_DURATION: Duration = Duration::from_secs(5);

/// The usage line shown with an argument error.
pub const USAGE: &str = "dsu_probe [port 1-65535] [seconds > 0]";

/// What to probe and for how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeOptions {
    pub port: Port,
    pub duration: Duration,
}

impl ProbeOptions {
    pub fn parse(args: &[String]) -> Result<Self, ArgError> {
        let values = positional(args);
        Ok(Self {
            port: parse_port(values.first().copied(), "port", Port::DEFAULT)?,
            duration: positive_duration(values.get(1).copied(), "seconds", DEFAULT_DURATION)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::probe::ProbeOptions;
    use dsu::Port;

    const OTHER_PORT: u16 = 26761;

    #[test]
    fn defaults_and_overrides() {
        let defaults = ProbeOptions::parse(&[]).map(|options| options.port);
        assert_eq!(defaults, Ok(Port::DEFAULT));
        let given = ProbeOptions::parse(&["26761".to_owned(), "2".to_owned()]);
        assert_eq!(
            given,
            Ok(ProbeOptions {
                port: Port::new(OTHER_PORT).unwrap_or(Port::DEFAULT),
                duration: Duration::from_secs(2),
            })
        );
        assert!(ProbeOptions::parse(&["0".to_owned()]).is_err());
    }
}
