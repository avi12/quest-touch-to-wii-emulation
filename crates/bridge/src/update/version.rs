//! Release versions as the tags name them (`v1.2.3`), ordered.

use std::fmt::{self, Display, Formatter};

/// A `major.minor.patch` version; field order makes the derived ordering numeric.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// This build's version, from `Cargo.toml`.
    pub fn current() -> Option<Self> {
        Self::parse(env!("CARGO_PKG_VERSION"))
    }

    /// `1.2.3` or a tag `v1.2.3`; `None` for anything else.
    pub fn parse(text: &str) -> Option<Self> {
        let numbers = text.strip_prefix('v').unwrap_or(text);
        let mut parts = numbers.split('.').map(str::parse::<u32>);
        let (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch)), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    /// The release tag for this version.
    pub fn tag(self) -> String {
        format!("v{self}")
    }
}

impl Display for Version {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use crate::update::version::Version;

    const MINOR: u32 = 10;

    #[test]
    fn parses_tags_and_plain_versions() {
        let expected = Version {
            major: 1,
            minor: MINOR,
            patch: 2,
        };
        assert_eq!(Version::parse("v1.10.2"), Some(expected));
        assert_eq!(Version::parse("1.10.2"), Some(expected));
        assert_eq!(expected.tag(), "v1.10.2");
        assert!(Version::current().is_some());
    }

    #[test]
    fn rejects_malformed() {
        for text in ["", "v1.2", "1.2.3.4", "v1.x.3", "latest"] {
            assert_eq!(Version::parse(text), None, "{text}");
        }
    }

    #[test]
    fn orders_numerically() {
        let older = Version::parse("1.9.0");
        let newer = Version::parse("1.10.0");
        assert!(newer > older);
    }
}
