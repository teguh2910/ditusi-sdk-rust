use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use super::common::Metadata;

/// Product availability status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductStatus {
    /// Product is in stock and available for immediate ordering.
    Available,
    /// Product is currently out of stock.
    OutOfStock,
    /// Custom or unexpected status value.
    Other(String),
}

impl Serialize for ProductStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Available => serializer.serialize_str("available"),
            Self::OutOfStock => serializer.serialize_str("out of stock"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for ProductStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "available" => Ok(Self::Available),
            "out of stock" => Ok(Self::OutOfStock),
            _ => Ok(Self::Other(s)),
        }
    }
}

impl fmt::Display for ProductStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "available"),
            Self::OutOfStock => write!(f, "out of stock"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Product item available for purchase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    /// Name of the parent game.
    #[serde(rename = "game_name", default)]
    pub game_name: Option<String>,
    /// Unique product identifier / SKU code (e.g. "ML5_0-S1").
    pub code: String,
    /// Display name of the product.
    pub name: String,
    /// Detailed description of the product.
    #[serde(default)]
    pub description: Option<String>,
    /// Current price in IDR.
    pub price: i64,
    /// Availability status ("available" or "out of stock").
    pub status: ProductStatus,
    /// Applicable region code (e.g. "ID").
    #[serde(default)]
    pub region: Option<String>,
}

/// Query parameters for listing products via `/product`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListProductsQuery {
    /// Filter products by game code (e.g. "JSX-J01").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_code: Option<String>,

    /// Fetch a single specific product by its product code (e.g. "ML5_0-S1").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_code: Option<String>,
}

impl ListProductsQuery {
    /// Creates a query filtered by game code.
    pub fn by_game(game_code: impl Into<String>) -> Self {
        Self {
            game_code: Some(game_code.into()),
            product_code: None,
        }
    }

    /// Creates a query filtered by product code.
    pub fn by_product(product_code: impl Into<String>) -> Self {
        Self {
            game_code: None,
            product_code: Some(product_code.into()),
        }
    }
}

/// Response returned by the `/product` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListProductsResponse {
    /// Status code (20 = success).
    pub status_code: i32,
    /// Status message (e.g. "SUCCESS").
    pub message: String,
    /// List of products matching the query.
    pub data: Vec<Product>,
    /// Response metadata (count, timestamp, filters).
    #[serde(default)]
    pub metadata: Option<Metadata>,
}
