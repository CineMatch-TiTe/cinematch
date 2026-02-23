//! Application configuration loaded once from environment variables.
//!
//! Uses `LazyLock` so env vars are read exactly once on first access.
//! All crates can use `Config::get()` to access shared configuration.

use secrecy::SecretString;
use std::sync::LazyLock;

/// Global application config, initialized on first access.
static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);

pub static OCTOCRAB_BASE: LazyLock<octocrab::Octocrab> = LazyLock::new(|| {
    octocrab::Octocrab::builder()
        .base_uri("https://github.com")
        .expect("Failed to initialize Octocrab builder")
        .build()
        .expect("Failed to build Octocrab client")
});

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Round 1 Voting phase timeout in seconds (env: `VOTING_R1_TIMEOUT_SECS`, default: 30)
    pub voting_r1_timeout_secs: u32,

    /// Round 2 Voting phase timeout in seconds (env: `VOTING_R2_TIMEOUT_SECS`, default: 20)
    pub voting_r2_timeout_secs: u32,

    /// Watching phase timeout in seconds (env: `WATCHING_TIMEOUT_SECS`, default: 900)
    pub watching_timeout_secs: u32,

    /// Ready countdown duration in seconds (env: `READY_COUNTDOWN_SECS`, default: 5.0)
    pub ready_countdown_secs: f32,
}

/// Shared application configuration.
///
/// Values are read from environment variables. Essential values are required,
/// while GitHub OAuth is optional.
#[derive(Debug, Clone)]
pub struct Config {
    // ── Timeouts ──────────────────────────────────────────────
    pub timeouts: TimeoutConfig,

    // ── Databases ─────────────────────────────────────────────
    /// PostgreSQL connection URL (env: `DATABASE_URL`, required)
    pub database_url: SecretString,

    /// Redis connection URL (env: `REDIS_URL`, required)
    pub redis_url: SecretString,

    /// Qdrant vector database URL (env: `QDRANT_URL`, required)
    pub qdrant_url: String,

    // ── Server ────────────────────────────────────────────────
    /// Server host (env: `SERVER_HOST`, default: 0.0.0.0)
    pub server_host: String,

    /// Server port (env: `SERVER_PORT`, default: 8080)
    pub server_port: u16,

    /// Secret token for sessions and JWT (env: `SECRET_TOKEN`, required, >= 64 bytes)
    pub secret_token: SecretString,

    // ── GitHub OAuth ──────────────────────────────────────────
    /// Optional GitHub OAuth configuration.
    pub github: Option<GithubConfig>,
}

/// GitHub-specific configuration.
#[derive(Debug, Clone)]
pub struct GithubConfig {
    /// GitHub Client ID (env: `GITHUB_CLIENT_ID`)
    pub client_id: String,

    /// GitHub Client Secret (env: `GITHUB_CLIENT_SECRET`)
    pub client_secret: SecretString,
}

impl Config {
    /// Get the global config instance (lazily initialized from env).
    pub fn get() -> &'static Config {
        &CONFIG
    }

    /// Build config from environment variables, falling back to defaults.
    fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
        let qdrant_url = std::env::var("QDRANT_URL").expect("QDRANT_URL must be set");
        let secret_token = std::env::var("SECRET_TOKEN").expect("SECRET_TOKEN must be set");

        if secret_token.len() < 64 {
            panic!("SECRET_TOKEN must be at least 64 bytes for secure session management");
        }

        let github = match (
            std::env::var("GITHUB_CLIENT_ID"),
            std::env::var("GITHUB_CLIENT_SECRET"),
        ) {
            (Ok(client_id), Ok(client_secret)) => Some(GithubConfig {
                client_id,
                client_secret: SecretString::from(client_secret),
            }),
            _ => {
                log::info!("GitHub OAuth config is incomplete; GitHub login will be disabled");
                None
            }
        };

        Self {
            timeouts: TimeoutConfig {
                voting_r1_timeout_secs: env_parse("VOTING_R1_TIMEOUT_SECS", 30),
                voting_r2_timeout_secs: env_parse("VOTING_R2_TIMEOUT_SECS", 20),
                watching_timeout_secs: env_parse("WATCHING_TIMEOUT_SECS", 900),
                ready_countdown_secs: env_parse("READY_COUNTDOWN_SECS", 5.0),
            },
            database_url: SecretString::from(database_url),
            redis_url: SecretString::from(redis_url),
            qdrant_url,
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env_parse("SERVER_PORT", 8080),
            secret_token: SecretString::from(secret_token),
            github,
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
