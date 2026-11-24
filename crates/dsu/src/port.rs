//! The UDP port a DSU server listens on.

/// A DSU server port, 1..65535 (never 0, which would mean "any port").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Port(u16);

impl Port {
    /// Dolphin's default DSU server port; the bridge serves the right controller on it.
    pub const DEFAULT: Self = Self(26760);

    /// The port `number`, or `None` for 0.
    pub const fn new(number: u16) -> Option<Self> {
        match number {
            0 => None,
            port => Some(Self(port)),
        }
    }

    /// The port number.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::Port;

    #[test]
    fn rejects_zero() {
        assert_eq!(Port::new(0), None);
        assert_eq!(Port::new(Port::DEFAULT.get()), Some(Port::DEFAULT));
    }
}
