//! Utility functions for party configuration.

/// Voting and watching timeout durations (seconds).
/// Reads from the shared `Config` (LazyLock, initialized once from env).
pub fn get_timeout_secs() -> (u32, u32) {
    let cfg = cinematch_common::Config::get();
    (cfg.voting_timeout_secs, cfg.watching_timeout_secs)
}
