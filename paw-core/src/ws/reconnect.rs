use std::time::Duration;

const DEFAULT_MAX_ATTEMPTS: usize = 10;
const DEFAULT_RETRY_DELAYS_SECS: &[u64] = &[1, 2, 4, 8, 16, 30];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconnectionManager {
    max_attempts: usize,
    retry_delays: Vec<Duration>,
    attempts: usize,
}

impl Default for ReconnectionManager {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ATTEMPTS, default_retry_delays())
    }
}

impl ReconnectionManager {
    pub fn new(max_attempts: usize, retry_delays: Vec<Duration>) -> Self {
        Self {
            max_attempts,
            retry_delays: if retry_delays.is_empty() {
                default_retry_delays()
            } else {
                retry_delays
            },
            attempts: 0,
        }
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }

    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    pub fn next_delay(&mut self) -> Option<Duration> {
        if !self.can_retry() {
            return None;
        }

        let index = self.attempts.min(self.retry_delays.len().saturating_sub(1));
        self.attempts += 1;
        self.retry_delays.get(index).copied()
    }

    pub fn on_connected(&mut self) {
        self.attempts = 0;
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }
}

fn default_retry_delays() -> Vec<Duration> {
    DEFAULT_RETRY_DELAYS_SECS
        .iter()
        .copied()
        .map(Duration::from_secs)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backs_off_and_caps_to_last_delay() {
        let mut manager = ReconnectionManager::default();

        let delays: Vec<_> = (0..7).filter_map(|_| manager.next_delay()).collect();

        assert_eq!(delays[0], Duration::from_secs(1));
        assert_eq!(delays[1], Duration::from_secs(2));
        assert_eq!(delays[2], Duration::from_secs(4));
        assert_eq!(delays[5], Duration::from_secs(30));
        assert_eq!(delays[6], Duration::from_secs(30));
    }

    #[test]
    fn stops_after_max_attempts_until_reset() {
        let mut manager = ReconnectionManager::new(2, vec![Duration::from_secs(1)]);

        assert_eq!(manager.next_delay(), Some(Duration::from_secs(1)));
        assert_eq!(manager.next_delay(), Some(Duration::from_secs(1)));
        assert_eq!(manager.next_delay(), None);

        manager.on_connected();
        assert_eq!(manager.next_delay(), Some(Duration::from_secs(1)));
    }
}
