use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use crate::error::Result;
use crate::models::auth::AccessTokenResponse;

/// Calculates the SHA-256 hex string for the given raw input.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generates a current ISO 8601 timestamp with numeric timezone offset (e.g. `2024-01-01T00:00:00+00:00`).
pub fn current_timestamp() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// Calculates the signature for the `/access-token` request.
///
/// Formula: `sha256(client_key + ":" + timestamp)`
pub fn calculate_token_signature(client_key: &str, timestamp: &str) -> String {
    let raw = format!("{}:{}", client_key, timestamp);
    sha256_hex(&raw)
}

/// Calculates the signature for standard API requests (GET/POST).
///
/// Formula: `sha256(path + ":" + client_key + ":" + timestamp + ":" + compact_json_body)`
///
/// - `path`: Full path including prefix, e.g. `/api/dev/v1/product`
/// - `client_key`: Your client key / secret
/// - `timestamp`: ISO 8601 formatted timestamp matching the `X-TIMESTAMP` header
/// - `compact_json_body`: Compact JSON string (no spaces). For GET requests with query parameters,
///   this is the serialized query object (e.g. `{"gameCode":"XYZ"}`); without query parameters, this is `{}`.
pub fn calculate_api_signature(
    path: &str,
    client_key: &str,
    timestamp: &str,
    compact_json_body: &str,
) -> String {
    let raw = format!("{}:{}:{}:{}", path, client_key, timestamp, compact_json_body);
    sha256_hex(&raw)
}

/// Cached Bearer token with expiration time.
#[derive(Debug, Clone)]
pub struct CachedToken {
    /// Bearer access token string
    pub token: String,
    /// Absolute expiration timestamp
    pub expires_at: DateTime<Utc>,
}

impl CachedToken {
    /// Checks if the cached token is still valid, applying a 60-second safety margin.
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        // Treat token as expired 60 seconds before actual expiry to prevent race conditions
        self.expires_at > (now + Duration::seconds(60))
    }
}

/// Thread-safe manager for obtaining, caching, and auto-refreshing Bearer access tokens.
#[derive(Debug, Clone)]
pub struct TokenManager {
    cache: Arc<RwLock<Option<CachedToken>>>,
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenManager {
    /// Creates a new, empty [`TokenManager`].
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Creates a [`TokenManager`] pre-seeded with a static token that never expires.
    pub fn with_static_token(token: impl Into<String>) -> Self {
        let cached = CachedToken {
            token: token.into(),
            expires_at: DateTime::<Utc>::MAX_UTC,
        };
        Self {
            cache: Arc::new(RwLock::new(Some(cached))),
        }
    }

    /// Returns a valid cached token if available and unexpired.
    pub async fn get_cached_token(&self) -> Option<String> {
        let read = self.cache.read().await;
        if let Some(cached) = &*read {
            if cached.is_valid() {
                return Some(cached.token.clone());
            }
        }
        None
    }

    /// Updates the cached token with a fresh response from `/access-token`.
    pub async fn update_from_response(&self, res: &AccessTokenResponse) -> Result<String> {
        let expiry_seconds = res.expiry_time.unwrap_or(600);
        let expires_at = Utc::now() + Duration::seconds(expiry_seconds as i64);

        let cached = CachedToken {
            token: res.access_token.clone(),
            expires_at,
        };

        let mut write = self.cache.write().await;
        *write = Some(cached);

        Ok(res.access_token.clone())
    }

    /// Manually invalidates the current cached token, forcing a refresh on next call.
    pub async fn invalidate(&self) {
        let mut write = self.cache.write().await;
        *write = None;
    }
}
