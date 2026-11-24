//! Clients that asked for pad data, dropped after a quiet spell.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Subscribed clients by address, with when each last asked.
#[derive(Debug)]
pub(crate) struct ClientTable {
    last_seen: HashMap<SocketAddr, Instant>,
    timeout: Duration,
}

impl ClientTable {
    pub(crate) fn new(timeout: Duration) -> Self {
        Self {
            last_seen: HashMap::new(),
            timeout,
        }
    }

    /// Records a pad request from `addr` at `now`.
    pub(crate) fn touch(&mut self, addr: SocketAddr, now: Instant) {
        self.last_seen.insert(addr, now);
    }

    /// Drops clients silent for longer than the timeout and returns the rest.
    pub(crate) fn live(&mut self, now: Instant) -> Vec<SocketAddr> {
        let timeout = self.timeout;
        self.last_seen
            .retain(|_addr, seen| now.saturating_duration_since(*seen) <= timeout);
        self.last_seen.keys().copied().collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.last_seen.len()
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::time::{Duration, Instant};

    use crate::server::clients::ClientTable;

    const TIMEOUT: Duration = Duration::from_secs(5);
    const SECOND_ASKS_AT: Duration = Duration::from_secs(3);
    const JUST_PAST_TIMEOUT: Duration = Duration::from_millis(5001);
    const SECOND_ASKS_AGAIN_AT: Duration = Duration::from_secs(9);
    const LAST_CHECK_AT: Duration = Duration::from_secs(10);

    #[test]
    fn drops_clients_after_timeout() {
        let start = Instant::now();
        let first: SocketAddr = "127.0.0.1:1000".parse().unwrap();
        let second: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let mut table = ClientTable::new(TIMEOUT);
        table.touch(first, start);
        table.touch(second, start + SECOND_ASKS_AT);
        assert_eq!(table.live(start + TIMEOUT).len(), 2);
        assert_eq!(table.live(start + JUST_PAST_TIMEOUT), vec![second]);
        table.touch(second, start + SECOND_ASKS_AGAIN_AT);
        assert_eq!(table.live(start + LAST_CHECK_AT), vec![second]);
        assert_eq!(table.len(), 1);
    }
}
