use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use super::common::Metadata;

/// Current status of a transaction / order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Order accepted and balance deducted, awaiting fulfillment.
    PendingOrder,
    /// Top-up is currently in progress.
    Process,
    /// Top-up completed successfully.
    Success,
    /// Order failed and amount was refunded to reseller balance.
    Refunded,
    /// Other or custom status string.
    Other(String),
}

impl Serialize for TransactionStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::PendingOrder => serializer.serialize_str("PENDING ORDER"),
            Self::Process => serializer.serialize_str("PROCESS"),
            Self::Success => serializer.serialize_str("SUCCESS"),
            Self::Refunded => serializer.serialize_str("REFUNDED"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for TransactionStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_uppercase().as_str() {
            "PENDING ORDER" => Ok(Self::PendingOrder),
            "PROCESS" => Ok(Self::Process),
            "SUCCESS" => Ok(Self::Success),
            "REFUNDED" => Ok(Self::Refunded),
            _ => Ok(Self::Other(s)),
        }
    }
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PendingOrder => write!(f, "PENDING ORDER"),
            Self::Process => write!(f, "PROCESS"),
            Self::Success => write!(f, "SUCCESS"),
            Self::Refunded => write!(f, "REFUNDED"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Request payload to create a new top-up order via `POST /transaction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Unique product identifier (e.g. "ML5_0-S1").
    pub product_code: String,

    /// Quantity of the product to purchase (minimum 1).
    pub amount: u32,

    /// Unique reference ID from the reseller system to identify the transaction and prevent duplicates.
    pub transaction_reference_id: String,

    /// User information required for the top-up (e.g. user_id, zone_id, server).
    pub form_details: serde_json::Value,

    /// Optional expected price for price protection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_price: Option<u64>,

    /// End user's IP address. Mandatory for Hoyoverse games (Genshin Impact, Zenless Zone Zero, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_user_ip_address: Option<String>,
}

impl TransactionRequest {
    /// Creates a new order request builder.
    pub fn builder(
        product_code: impl Into<String>,
        transaction_reference_id: impl Into<String>,
        form_details: serde_json::Value,
    ) -> TransactionRequestBuilder {
        TransactionRequestBuilder::new(product_code, transaction_reference_id, form_details)
    }
}

/// Builder for constructing [`TransactionRequest`].
#[derive(Debug, Clone)]
pub struct TransactionRequestBuilder {
    product_code: String,
    amount: u32,
    transaction_reference_id: String,
    form_details: serde_json::Value,
    initial_price: Option<u64>,
    end_user_ip_address: Option<String>,
}

impl TransactionRequestBuilder {
    /// Initializes builder with mandatory fields.
    pub fn new(
        product_code: impl Into<String>,
        transaction_reference_id: impl Into<String>,
        form_details: serde_json::Value,
    ) -> Self {
        Self {
            product_code: product_code.into(),
            amount: 1,
            transaction_reference_id: transaction_reference_id.into(),
            form_details,
            initial_price: None,
            end_user_ip_address: None,
        }
    }

    /// Sets order quantity (defaults to 1).
    pub fn amount(mut self, amount: u32) -> Self {
        self.amount = amount;
        self
    }

    /// Sets expected price for price protection.
    pub fn initial_price(mut self, price: u64) -> Self {
        self.initial_price = Some(price);
        self
    }

    /// Sets end user IP address (mandatory for Hoyoverse titles).
    pub fn end_user_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.end_user_ip_address = Some(ip.into());
        self
    }

    /// Builds the [`TransactionRequest`].
    pub fn build(self) -> TransactionRequest {
        TransactionRequest {
            product_code: self.product_code,
            amount: self.amount,
            transaction_reference_id: self.transaction_reference_id,
            form_details: self.form_details,
            initial_price: self.initial_price,
            end_user_ip_address: self.end_user_ip_address,
        }
    }
}

/// Response returned from `POST /transaction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderResponse {
    /// Status code (21 = transaction created / pending, 30+ = error).
    pub status_code: i32,

    /// Status message (e.g. "SUCCESS").
    pub message: String,

    /// Current order status ("PENDING ORDER", "PROCESS", "SUCCESS", "REFUNDED").
    #[serde(default)]
    pub status_transaction: Option<TransactionStatus>,

    /// Unique transaction ID generated by DITUSI (e.g. "D2C-30330245313890503").
    #[serde(default)]
    pub transaction_id: Option<String>,

    /// Original reseller reference ID echoed back from request.
    #[serde(default)]
    pub original_transaction_reference_id: Option<String>,
}

/// Product details embedded inside a transaction check response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionProduct {
    /// Product code.
    pub code: String,
    /// Product display name.
    pub name: String,
    /// Quantity ordered.
    pub amount: i64,
}

/// Detailed transaction record returned by `GET /transaction/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionDetails {
    /// Order status.
    pub status: TransactionStatus,

    /// DITUSI transaction ID.
    #[serde(rename = "transactionId")]
    pub transaction_id: String,

    /// Reseller reference ID (may be null).
    #[serde(rename = "transactionReferenceId", default)]
    pub transaction_reference_id: Option<String>,

    /// Total gross amount charged for the transaction.
    #[serde(alias = "grossAmount", alias = "gross_amount", default)]
    pub gross_amount: Option<f64>,

    /// Form details provided when placing the order.
    #[serde(alias = "formDetails", alias = "form_details", default)]
    pub form_details: Option<serde_json::Value>,

    /// Information about the ordered product.
    #[serde(default)]
    pub product: Option<TransactionProduct>,

    /// Error message if the order failed or was refunded.
    #[serde(alias = "errorMessage", alias = "error_message", default)]
    pub error_message: Option<String>,
}

/// Response returned from `GET /transaction/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionResponse {
    /// Status code (20 = success).
    pub status_code: i32,

    /// Status message (e.g. "SUCCESS").
    pub message: String,

    /// Transaction details.
    pub data: TransactionDetails,

    /// Response metadata.
    #[serde(default)]
    pub metadata: Option<Metadata>,
}
