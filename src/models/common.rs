use serde::{Deserialize, Serialize};

/// Generic metadata returned in various DITUSI API responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Timestamp of response generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Number of items returned in the payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,

    /// Query filters applied to the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

/// Standard error response format returned by DITUSI API endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error status code (e.g. 30, 40, 45, 50).
    pub status_code: i32,
    /// Detailed error message or description.
    pub message: String,
}

/// Request payload for sandbox webhook callback testing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxWebhookTestRequest {
    /// Destination webhook URL where DITUSI sandbox will dispatch test callbacks.
    pub url: String,

    /// Test transaction ID (for transaction callback tests).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    /// Test product code (for product callback tests).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_code: Option<String>,
}

/// Response payload from sandbox webhook callback testing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxWebhookTestResponse {
    /// Response status code (20 = success).
    pub status_code: i32,
    /// Status message (e.g. "SUCCESS").
    pub message: String,
    /// Confirmation details.
    pub data: serde_json::Value,
}
