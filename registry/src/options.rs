use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Options {
    pub registration_timeout: Duration
}

impl Options {
    pub fn from_env() -> Self {
        Self {
            registration_timeout: get_registration_timeout_from_env(),
        }
    }
}

fn get_registration_timeout_from_env() -> Duration {
    Duration::from_secs(
        std::env::var("REGISTRATION_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .expect("Invalid REGISTRATION_TIMEOUT"),
    )
}