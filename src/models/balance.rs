use serde::{Deserialize, Serialize};
use super::common::Metadata;

/// Reseller account balance information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceData {
    /// Reseller / Partner identifier.
    #[serde(alias = "partnerId", alias = "partner_id")]
    pub partner_id: i64,

    /// Reseller tier description (e.g. "RESELLER GOLD").
    #[serde(alias = "partnerType", alias = "partner_type")]
    pub partner_type: String,

    /// Available deposit balance in IDR.
    #[serde(alias = "DepositBalance", alias = "depositBalance", alias = "deposit_balance")]
    pub deposit_balance: i64,
}

/// Response returned from `GET /balance`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// Status code (20 = success).
    pub status_code: i32,
    /// Status message (e.g. "SUCCESS").
    pub message: String,
    /// Reseller account balance details.
    pub data: BalanceData,
    /// Response metadata.
    #[serde(default)]
    pub metadata: Option<Metadata>,
}
