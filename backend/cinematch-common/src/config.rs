//! Application configuration loaded once from environment variables.
//!
//! Uses `LazyLock` so env vars are read exactly once on first access.
//! All crates can use `Config::get()` to access shared configuration.

use std::sync::LazyLock;

/// Global application config, initialized on first access.
static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);

/// Shared application configuration.
///
/// Values are read from environment variables with sensible defaults.
/// Access via `Config::get()` from anywhere in the codebase.
#[derive(Debug, Clone)]
pub struct Config {
    // ── Timeouts ──────────────────────────────────────────────
    /// Voting phase timeout in seconds (env: `VOTING_TIMEOUT_SECS`, default: 180)
    pub voting_timeout_secs: u32,

    /// Watching phase timeout in seconds (env: `WATCHING_TIMEOUT_SECS`, default: 900)
    pub watching_timeout_secs: u32,

    /// Ready countdown duration in seconds (env: `READY_COUNTDOWN_SECS`, default: 5.0)
    pub ready_countdown_secs: f32,
}

impl Config {
    /// Get the global config instance (lazily initialized from env).
    pub fn get() -> &'static Config {
        &CONFIG
    }

    /// Build config from environment variables, falling back to defaults.
    fn from_env() -> Self {
        Self {
            voting_timeout_secs: env_parse("VOTING_TIMEOUT_SECS", 180),
            watching_timeout_secs: env_parse("WATCHING_TIMEOUT_SECS", 900),
            ready_countdown_secs: env_parse("READY_COUNTDOWN_SECS", 5.0),
        }
    }
}

/// Parse an env var into `T`, returning `default` on missing or bad value.
fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
