//! A UDP port as the command line takes it: 1-65535.

use dsu::Port;

use crate::cli::ArgError;

/// The port given as `raw`, or `default` when the argument is absent.
pub fn parse_port(raw: Option<&str>, name: &str, default: Port) -> Result<Port, ArgError> {
    let Some(text) = raw else {
        return Ok(default);
    };
    text.trim()
        .parse::<u16>()
        .ok()
        .and_then(Port::new)
        .ok_or_else(|| ArgError::new(name, "an integer 1-65535", text))
}

#[cfg(test)]
mod tests {
    use dsu::Port;

    use crate::cli::parse_port;

    const OTHER_PORT: u16 = 26761;

    #[test]
    fn accepts_1_to_65535() {
        assert_eq!(
            parse_port(Some("26761"), "port", Port::DEFAULT).map(Port::get),
            Ok(OTHER_PORT)
        );
        assert_eq!(parse_port(None, "port", Port::DEFAULT), Ok(Port::DEFAULT));
        assert!(parse_port(Some("0"), "port", Port::DEFAULT).is_err());
        assert!(parse_port(Some("70000"), "port", Port::DEFAULT).is_err());
    }
}
