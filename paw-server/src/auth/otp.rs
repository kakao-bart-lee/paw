use chrono::{Duration, Utc};
use rand::Rng;

pub fn generate_otp() -> String {
    if let Some(code) = fixed_otp() {
        return code;
    }

    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000))
}

pub fn fixed_otp() -> Option<String> {
    let code = std::env::var("PAW_FIXED_OTP").ok()?;
    parse_fixed_otp(&code)
}

fn parse_fixed_otp(code: &str) -> Option<String> {
    if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
        Some(code.to_string())
    } else {
        None
    }
}

pub fn otp_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::minutes(5)
}

#[cfg(test)]
mod tests {
    use super::parse_fixed_otp;

    #[test]
    fn fixed_otp_is_used_when_valid() {
        assert_eq!(parse_fixed_otp("137900").as_deref(), Some("137900"));
    }

    #[test]
    fn invalid_fixed_otp_is_ignored() {
        assert_eq!(parse_fixed_otp("abc"), None);
        assert_eq!(parse_fixed_otp("12345"), None);
        assert_eq!(parse_fixed_otp("1234567"), None);
    }
}
