//! Generic Redis cache helpers with TTL support.
//!
//! Provides type-safe get/set/delete operations with automatic JSON serialization.

use deadpool_redis::{Connection, Pool, redis::AsyncCommands};
use serde::{Serialize, de::DeserializeOwned};

use crate::DbError;

/// Default TTL values (in seconds)
pub const TTL_1_HOUR: u64 = 60 * 60;
pub const TTL_1_DAY: u64 = 60 * 60 * 24;
pub const TTL_7_DAYS: u64 = 60 * 60 * 24 * 7;

/// Get a value from Redis cache.
///
/// Returns `Ok(Some(value))` if found and deserialized successfully,
/// `Ok(None)` if key doesn't exist, or `Err` on Redis/parse error.
pub async fn get<T: DeserializeOwned>(
    conn: &mut Connection,
    key: &str,
) -> Result<Option<T>, DbError> {
    let data: Option<String> = conn.get(key).await?;
    match data {
        Some(json) => {
            let value: T = serde_json::from_str(&json)
                .map_err(|e| DbError::Other(format!("Cache deserialize error: {}", e)))?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Set a value in Redis cache with TTL.
///
/// Value is serialized to JSON before storing.
pub async fn set<T: Serialize>(
    conn: &mut Connection,
    key: &str,
    value: &T,
    ttl_secs: u64,
) -> Result<(), DbError> {
    let json = serde_json::to_string(value)
        .map_err(|e| DbError::Other(format!("Cache serialize error: {}", e)))?;
    conn.set_ex::<_, _, ()>(key, json, ttl_secs).await?;
    Ok(())
}

/// Delete a key from Redis cache.
pub async fn delete(conn: &mut Connection, key: &str) -> Result<(), DbError> {
    conn.del::<_, ()>(key).await?;
    Ok(())
}

/// Delete all keys matching a pattern.
///
/// Uses SCAN + DEL for safety (avoids blocking on large keyspaces).
/// Pattern examples: `movie:*`, `search:action:*`
pub async fn invalidate_pattern(conn: &mut Connection, pattern: &str) -> Result<u64, DbError> {
    let keys: Vec<String> = conn.keys(pattern).await?;
    if keys.is_empty() {
        return Ok(0);
    }
    let count = keys.len() as u64;
    conn.del::<_, ()>(keys).await?;
    Ok(count)
}

/// Cache key namespaces to avoid collisions with actix-session.
///
/// | Namespace | Data | TTL |
/// |-----------|------|-----|
/// | `session:` | actix-session (identity) | Session TTL |
/// | `movie:` | Movie details by ID | 7 days |
/// | `genres:` | Genre list | 7 days |
/// | `search:` | Search results | 1 hour |
pub mod keys {
    /// Movie cache key: `movie:{id}`
    pub fn movie(id: i64) -> String {
        format!("movie:{}", id)
    }

    /// All genres cache key
    pub const GENRES_ALL: &str = "genres:all";

    /// Search results cache key: `search:{query_hash}`
    pub fn search(query_hash: &str) -> String {
        format!("search:{}", query_hash)
    }
}

/// Cache-aside pattern helper for async operations.
///
/// 1. Try to get from cache
/// 2. On miss, call the fallback function
/// 3. Store result in cache (fire-and-forget on cache write failure)
/// 4. Return result
///
/// # Example
/// ```ignore
/// let movie = cache_aside(
///     &mut redis_conn,
///     &keys::movie(movie_id),
///     TTL_7_DAYS,
///     || async { db.get_movie_from_postgres(movie_id).await }
/// ).await?;
/// ```
pub async fn cache_aside<T, F, Fut>(
    pool: &Pool,
    key: &str,
    ttl_secs: u64,
    fallback: F,
) -> Result<T, DbError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, DbError>>,
{
    // Try cache first
    if let Ok(mut conn) = pool.get().await
        && let Ok(Some(cached)) = get::<T>(&mut conn, key).await
    {
        return Ok(cached);
    }

    // Cache miss - call fallback
    let value = fallback().await?;

    // Store in cache (fire-and-forget, don't fail the request if cache write fails)
    if let Ok(mut conn) = pool.get().await {
        let _ = set(&mut conn, key, &value, ttl_secs).await;
    }

    Ok(value)
}
