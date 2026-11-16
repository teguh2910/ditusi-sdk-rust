use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::models::common::Metadata;
use crate::models::transaction::{TransactionProduct, TransactionStatus};

/// Payload sent to the registered webhook endpoint on order status updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWebhookData {
    /// New transaction status.
    pub status: TransactionStatus,
    /// DITUSI unique transaction ID.
    pub transaction_id: String,
    /// Reseller reference ID.
    #[serde(default)]
    pub transaction_reference_id: Option<String>,
    /// Error message if the order failed.
    #[serde(default)]
    pub error_message: Option<String>,
    /// Information about the product.
    #[serde(default)]
    pub product: Option<TransactionProduct>,
}

/// Root payload for transaction webhook notifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWebhookPayload {
    /// Status code (20 = success).
    pub status_code: i32,
    /// Status message.
    pub message: String,
    /// Event data.
    pub data: TransactionWebhookData,
    /// Metadata including timestamp.
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

/// Payload sent to the registered webhook endpoint when product details or availability change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductWebhookData {
    /// Game title.
    #[serde(default)]
    pub game_name: Option<String>,
    /// Product SKU code.
    pub product_code: String,
    /// Display name of the product.
    pub product_name: String,
    /// Product description.
    #[serde(default)]
    pub description: Option<String>,
    /// Current price in IDR.
    pub price: i64,
    /// In-game currency name (e.g. "Diamonds").
    #[serde(default)]
    pub in_game_currency: Option<String>,
    /// Product status (e.g. "available", "out of stock").
    pub status: String,
}

/// Root payload for product change webhook notifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductWebhookPayload {
    /// Status code (20 = success).
    pub status_code: i32,
    /// Status message.
    pub message: String,
    /// Product event data.
    pub data: ProductWebhookData,
    /// Metadata including timestamp.
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

/// Generic webhook event that can be either a transaction update or product catalog change.
#[derive(Debug, Clone, PartialEq)]
pub enum WebhookEvent {
    /// Order status update notification.
    Transaction(TransactionWebhookPayload),
    /// Product catalog update notification.
    Product(ProductWebhookPayload),
    /// Unknown or unsupported webhook event.
    Unknown(serde_json::Value),
}

/// Verifies whether the incoming HTTP `Authorization` header matches the expected secret webhook verification token.
///
/// Supports tokens passed directly or formatted with `Bearer <token>`.
pub fn verify_webhook_token(auth_header: &str, expected_token: &str) -> bool {
    let clean_header = auth_header.trim();
    let token = if let Some(stripped) = clean_header.strip_prefix("Bearer ") {
        stripped.trim()
    } else {
        clean_header
    };

    token == expected_token.trim()
}

/// Parses raw webhook payload JSON into a strongly typed [`WebhookEvent`].
pub fn parse_webhook_event(raw_json: &str) -> Result<WebhookEvent> {
    let value: serde_json::Value = serde_json::from_str(raw_json)?;

    if let Some(data) = value.get("data") {
        if data.get("transactionId").is_some() {
            let payload: TransactionWebhookPayload = serde_json::from_value(value)?;
            return Ok(WebhookEvent::Transaction(payload));
        }

        if data.get("productCode").is_some() {
            let payload: ProductWebhookPayload = serde_json::from_value(value)?;
            return Ok(WebhookEvent::Product(payload));
        }
    }

    Ok(WebhookEvent::Unknown(value))
}
