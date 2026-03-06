use chrono::{Duration, Utc};
use rand::Rng;

pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000))
}

pub fn otp_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::minutes(5)
}
