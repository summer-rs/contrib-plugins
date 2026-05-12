//! summer Redis Storage for Sa-Token
//!
//! This module provides a storage implementation that reuses the Redis connection
//! from `summer-redis` plugin, avoiding duplicate connections.

use sa_token_adapter::storage::{SaStorage, StorageError, StorageResult};
use std::time::Duration;
use summer::async_trait;
use summer_redis::redis::AsyncCommands;
use summer_redis::Redis;

/// Redis storage implementation using summer-redis connection
///
/// This storage reuses the `Redis` (ConnectionManager) component from `summer-redis`,
/// so you don't need to configure a separate Redis connection for sa-token.
pub struct SummerRedisStorage {
    client: Redis,
    prefix: Option<String>,
    rewrite_prefix: bool,
}

impl SummerRedisStorage {
    /// Create a new SummerRedisStorage with the given Redis connection
    pub fn new(client: Redis, prefix: Option<String>, rewrite_prefix: bool) -> Self {
        let prefix = normalize_storage_prefix(prefix);
        Self {
            client,
            prefix,
            rewrite_prefix,
        }
    }
}

fn normalize_storage_prefix(prefix: Option<String>) -> Option<String> {
    match prefix {
        Some(prefix) if prefix.is_empty() => None,
        Some(prefix) if prefix.ends_with(':') => Some(prefix),
        Some(prefix) => Some(format!("{prefix}:")),
        None => None,
    }
}

fn apply_storage_prefix(prefix: Option<&str>, rewrite_prefix: bool, key: &str) -> String {
    match prefix {
        Some(prefix) if !prefix.is_empty() && rewrite_prefix => {
            // Upstream sa-token-core hardcodes the logical storage root as `sa:`
            // and does not expose a configuration hook for it, so the adapter can
            // only rewrite that prefix at the storage boundary.
            if let Some(stripped) = key.strip_prefix("sa:") {
                format!("{prefix}{stripped}")
            } else {
                format!("{prefix}{key}")
            }
        }
        Some(prefix) if !prefix.is_empty() => format!("{prefix}{key}"),
        _ => key.to_string(),
    }
}

fn apply_storage_prefix_to_pattern(
    prefix: Option<&str>,
    rewrite_prefix: bool,
    pattern: &str,
) -> String {
    apply_storage_prefix(prefix, rewrite_prefix, pattern)
}

fn strip_storage_prefix(prefix: Option<&str>, rewrite_prefix: bool, key: &str) -> String {
    match prefix {
        Some(prefix) if !prefix.is_empty() && rewrite_prefix => {
            if let Some(stripped) = key.strip_prefix(prefix) {
                // Restore the upstream logical `sa:` key shape before returning
                // values to sa-token-core, which still expects its hardcoded root.
                if stripped.starts_with("sa:") {
                    stripped.to_string()
                } else {
                    format!("sa:{stripped}")
                }
            } else {
                key.to_string()
            }
        }
        Some(prefix) if !prefix.is_empty() => key.strip_prefix(prefix).unwrap_or(key).to_string(),
        _ => key.to_string(),
    }
}

#[async_trait]
impl SaStorage for SummerRedisStorage {
    async fn get(&self, key: &str) -> StorageResult<Option<String>> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);
        tracing::debug!("SummerRedisStorage GET key: {}", key);
        conn.get(key)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> StorageResult<()> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);
        tracing::debug!("SummerRedisStorage SET key: {}", key);

        if let Some(ttl) = ttl {
            conn.set_ex(key, value, ttl.as_secs())
                .await
                .map_err(|e| StorageError::OperationFailed(e.to_string()))
        } else {
            conn.set(key, value)
                .await
                .map_err(|e| StorageError::OperationFailed(e.to_string()))
        }
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);

        conn.del(key)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);

        conn.exists(key)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn expire(&self, key: &str, ttl: Duration) -> StorageResult<()> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);

        conn.expire(key, ttl.as_secs() as i64)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn ttl(&self, key: &str) -> StorageResult<Option<Duration>> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);

        let ttl_secs: i64 = conn
            .ttl(key)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        match ttl_secs {
            -2 => Ok(None), // Key does not exist
            -1 => Ok(None), // Key exists but has no expiry
            secs if secs > 0 => Ok(Some(Duration::from_secs(secs as u64))),
            _ => Ok(Some(Duration::from_secs(0))),
        }
    }

    async fn mget(&self, keys: &[&str]) -> StorageResult<Vec<Option<String>>> {
        let mut conn = self.client.clone();
        let keys: Vec<String> = keys
            .iter()
            .map(|key| apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key))
            .collect();

        // Use mget command for multiple keys
        summer_redis::redis::cmd("MGET")
            .arg(keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn mset(&self, items: &[(&str, &str)], ttl: Option<Duration>) -> StorageResult<()> {
        let mut conn = self.client.clone();

        // Use pipeline for batch operations
        let mut pipe = summer_redis::redis::pipe();
        for (key, value) in items {
            let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);
            if let Some(ttl) = ttl {
                pipe.set_ex(key, *value, ttl.as_secs());
            } else {
                pipe.set(key, *value);
            }
        }

        pipe.query_async(&mut conn)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn mdel(&self, keys: &[&str]) -> StorageResult<()> {
        let mut conn = self.client.clone();
        let keys: Vec<String> = keys
            .iter()
            .map(|key| apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key))
            .collect();

        conn.del(keys)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn incr(&self, key: &str) -> StorageResult<i64> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);

        conn.incr(key, 1)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn decr(&self, key: &str) -> StorageResult<i64> {
        let mut conn = self.client.clone();
        let key = apply_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, key);

        conn.decr(key, 1)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))
    }

    async fn clear(&self) -> StorageResult<()> {
        let mut conn = self.client.clone();
        let pattern =
            apply_storage_prefix_to_pattern(self.prefix.as_deref(), self.rewrite_prefix, "sa:*");

        // Get all matching keys
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        if !keys.is_empty() {
            conn.del::<_, ()>(&keys)
                .await
                .map_err(|e| StorageError::OperationFailed(e.to_string()))?;
        }

        Ok(())
    }

    async fn keys(&self, pattern: &str) -> StorageResult<Vec<String>> {
        let mut conn = self.client.clone();
        let pattern =
            apply_storage_prefix_to_pattern(self.prefix.as_deref(), self.rewrite_prefix, pattern);

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        Ok(keys
            .into_iter()
            .map(|key| strip_storage_prefix(self.prefix.as_deref(), self.rewrite_prefix, &key))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_storage_prefix, apply_storage_prefix_to_pattern, normalize_storage_prefix,
        strip_storage_prefix,
    };

    #[test]
    fn normalizes_storage_prefix() {
        assert_eq!(
            normalize_storage_prefix(Some("demo".to_string())),
            Some("demo:".to_string())
        );
        assert_eq!(
            normalize_storage_prefix(Some("demo:".to_string())),
            Some("demo:".to_string())
        );
        assert_eq!(normalize_storage_prefix(Some(String::new())), None);
        assert_eq!(normalize_storage_prefix(None), None);
    }

    #[test]
    fn applies_prefix_to_key() {
        assert_eq!(
            apply_storage_prefix(Some("demo:"), false, "sa:token:abc"),
            "demo:sa:token:abc"
        );
        assert_eq!(
            apply_storage_prefix(Some("demo:"), true, "sa:token:abc"),
            "demo:token:abc"
        );
        assert_eq!(
            apply_storage_prefix(None, false, "sa:token:abc"),
            "sa:token:abc"
        );
    }

    #[test]
    fn applies_prefix_to_pattern() {
        assert_eq!(
            apply_storage_prefix_to_pattern(Some("demo:"), false, "sa:*"),
            "demo:sa:*"
        );
        assert_eq!(
            apply_storage_prefix_to_pattern(Some("demo:"), true, "sa:*"),
            "demo:*"
        );
        assert_eq!(apply_storage_prefix_to_pattern(None, false, "sa:*"), "sa:*");
    }

    #[test]
    fn strips_prefix_from_key() {
        assert_eq!(
            strip_storage_prefix(Some("demo:"), false, "demo:sa:token:abc"),
            "sa:token:abc"
        );
        assert_eq!(
            strip_storage_prefix(Some("demo:"), true, "demo:token:abc"),
            "sa:token:abc"
        );
        assert_eq!(
            strip_storage_prefix(Some("demo:"), true, "sa:token:abc"),
            "sa:token:abc"
        );
        assert_eq!(
            strip_storage_prefix(None, false, "sa:token:abc"),
            "sa:token:abc"
        );
    }
}
