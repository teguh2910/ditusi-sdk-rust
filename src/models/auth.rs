use serde::{Deserialize, Serialize};

/// Response returned from the `/access-token` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccessTokenResponse {
    /// Status code (20 = success).
    pub status_code: i32,
    /// Status message (e.g. "SUCCESS").
    pub message: String,
    /// JWT token string to include in the Authorization header.
    pub access_token: String,
    /// Token type (e.g. "Bearer").
    pub token_type: Option<String>,
    /// Token validity duration in seconds (defaults to 600 seconds).
    pub expiry_time: Option<u64>,
}
