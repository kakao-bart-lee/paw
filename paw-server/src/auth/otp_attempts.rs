use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

const DEFAULT_MAX_ATTEMPTS: u32 = 5;
const DEFAULT_LOCK_MINUTES: i64 = 15;
const DEFAULT_SUSPICIOUS_IP_THRESHOLD: u32 = 5;

#[derive(Clone)]
pub struct OtpAttemptGuard {
    inner: Arc<Mutex<AttemptState>>,
    max_attempts: u32,
    lock_duration: Duration,
    suspicious_ip_threshold: u32,
}

#[derive(Default)]
struct AttemptState {
    phone_attempts: HashMap<String, PhoneAttempt>,
    ip_attempts: HashMap<String, IpAttempt>,
}

struct PhoneAttempt {
    failed_attempts: u32,
    last_failed_at: DateTime<Utc>,
    locked_until: Option<DateTime<Utc>>,
}

struct IpAttempt {
    failed_attempts: u32,
    window_started_at: DateTime<Utc>,
}

impl Default for OtpAttemptGuard {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_ATTEMPTS,
            Duration::minutes(DEFAULT_LOCK_MINUTES),
        )
    }
}

impl OtpAttemptGuard {
    pub fn new(max_attempts: u32, lock_duration: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AttemptState::default())),
            max_attempts,
            lock_duration,
            suspicious_ip_threshold: DEFAULT_SUSPICIOUS_IP_THRESHOLD,
        }
    }

    pub fn retry_after_seconds(&self, phone: &str) -> Option<u64> {
        self.retry_after_seconds_at(phone, Utc::now())
    }

    pub fn record_failure(&self, phone: &str, ip: &str) -> Option<u64> {
        self.record_failure_at(phone, ip, Utc::now())
    }

    pub fn reset_phone(&self, phone: &str) {
        let mut state = self
            .inner
            .lock()
            .expect("otp attempt guard mutex should not be poisoned");
        state.phone_attempts.remove(phone);
    }

    fn retry_after_seconds_at(&self, phone: &str, now: DateTime<Utc>) -> Option<u64> {
        let mut state = self
            .inner
            .lock()
            .expect("otp attempt guard mutex should not be poisoned");

        let mut should_remove = false;
        let mut retry_after = None;

        if let Some(entry) = state.phone_attempts.get_mut(phone) {
            if let Some(locked_until) = entry.locked_until {
                if locked_until > now {
                    retry_after = Some(seconds_until(locked_until, now));
                } else {
                    should_remove = true;
                }
            } else if now - entry.last_failed_at >= self.lock_duration {
                should_remove = true;
            }
        }

        if should_remove {
            state.phone_attempts.remove(phone);
        }

        retry_after
    }

    fn record_failure_at(&self, phone: &str, ip: &str, now: DateTime<Utc>) -> Option<u64> {
        let mut state = self
            .inner
            .lock()
            .expect("otp attempt guard mutex should not be poisoned");

        let ip_attempts = self.bump_ip_failures(&mut state, ip, now);
        if ip_attempts >= self.suspicious_ip_threshold {
            tracing::warn!(
                ip = %ip,
                attempts = ip_attempts,
                "suspicious OTP verification failures from same IP"
            );
        }

        let entry = state
            .phone_attempts
            .entry(phone.to_string())
            .or_insert_with(|| PhoneAttempt {
                failed_attempts: 0,
                last_failed_at: now,
                locked_until: None,
            });

        if let Some(locked_until) = entry.locked_until {
            if locked_until > now {
                return Some(seconds_until(locked_until, now));
            }

            entry.failed_attempts = 0;
            entry.locked_until = None;
        }

        if now - entry.last_failed_at >= self.lock_duration {
            entry.failed_attempts = 0;
        }

        entry.failed_attempts += 1;
        entry.last_failed_at = now;

        if entry.failed_attempts >= self.max_attempts {
            let locked_until = now + self.lock_duration;
            entry.locked_until = Some(locked_until);
            return Some(seconds_until(locked_until, now));
        }

        None
    }

    fn bump_ip_failures(&self, state: &mut AttemptState, ip: &str, now: DateTime<Utc>) -> u32 {
        let entry = state
            .ip_attempts
            .entry(ip.to_string())
            .or_insert_with(|| IpAttempt {
                failed_attempts: 0,
                window_started_at: now,
            });

        if now - entry.window_started_at >= self.lock_duration {
            entry.failed_attempts = 0;
            entry.window_started_at = now;
        }

        entry.failed_attempts += 1;
        entry.failed_attempts
    }
}

pub fn otp_attempt_guard() -> &'static OtpAttemptGuard {
    static OTP_ATTEMPT_GUARD: LazyLock<OtpAttemptGuard> = LazyLock::new(OtpAttemptGuard::default);
    &OTP_ATTEMPT_GUARD
}

fn seconds_until(locked_until: DateTime<Utc>, now: DateTime<Utc>) -> u64 {
    let seconds = (locked_until - now).num_seconds();
    if seconds <= 0 {
        1
    } else {
        seconds as u64
    }
}

#[cfg(test)]
mod tests {
    use super::OtpAttemptGuard;
    use chrono::{Duration, Utc};

    #[test]
    fn success_after_valid_otp_resets_counter() {
        let guard = OtpAttemptGuard::new(5, Duration::minutes(15));
        let start = Utc::now();

        assert_eq!(
            guard.record_failure_at("+821012345678", "1.2.3.4", start),
            None
        );
        assert_eq!(
            guard.record_failure_at("+821012345678", "1.2.3.4", start + Duration::seconds(1)),
            None
        );

        guard.reset_phone("+821012345678");

        assert_eq!(guard.retry_after_seconds_at("+821012345678", start), None);
    }

    #[test]
    fn lock_after_five_failures() {
        let guard = OtpAttemptGuard::new(5, Duration::minutes(15));
        let start = Utc::now();

        for i in 0..4 {
            assert_eq!(
                guard.record_failure_at(
                    "+821055555555",
                    "9.9.9.9",
                    start + Duration::seconds(i as i64),
                ),
                None
            );
        }

        let retry_after = guard
            .record_failure_at("+821055555555", "9.9.9.9", start + Duration::seconds(4))
            .expect("fifth failure should lock");
        assert_eq!(retry_after, 900);
    }

    #[test]
    fn unlock_after_cooldown() {
        let guard = OtpAttemptGuard::new(5, Duration::minutes(15));
        let start = Utc::now();

        for i in 0..5 {
            guard.record_failure_at(
                "+821066666666",
                "8.8.8.8",
                start + Duration::seconds(i as i64),
            );
        }

        assert!(guard
            .retry_after_seconds_at("+821066666666", start + Duration::minutes(5))
            .is_some());

        assert_eq!(
            guard.retry_after_seconds_at("+821066666666", start + Duration::minutes(16)),
            None
        );
    }
}
