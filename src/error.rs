/// Errors that can occur when using the DITUSI SDK.
#[derive(Debug, thiserror::Error)]
pub enum DitusiError {
    /// An error returned by the underlying HTTP transport (reqwest).
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// An API-level error returned by DITUSI (e.g. invalid status code or business error).
    #[error("API error (status: {status_code}): {message}")]
    Api {
        /// DITUSI custom status code (e.g. 21, 30, 40, 45, etc.) or HTTP status
        status_code: i32,
        /// Message description returned by DITUSI
        message: String,
        /// Raw response body if available
        raw_body: Option<String>,
    },

    /// An error during authentication or token refresh.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid parameter supplied to a request or configuration.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Missing required header or field.
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    /// Webhook verification error (e.g. invalid token).
    #[error("Webhook verification failed: {0}")]
    WebhookVerificationFailed(String),
}

impl DitusiError {
    /// Creates a new API error.
    pub fn api(status_code: i32, message: impl Into<String>, raw_body: Option<String>) -> Self {
        Self::Api {
            status_code,
            message: message.into(),
            raw_body,
        }
    }
}

/// Convenience alias for SDK operations returning a `Result`.
pub type Result<T> = std::result::Result<T, DitusiError>;
