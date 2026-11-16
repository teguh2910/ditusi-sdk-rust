use serde::{Deserialize, Serialize};
use super::common::Metadata;

/// A form field requirement for placing orders in a game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormField {
    /// Name of the field expected in `formDetails` (e.g. "user_id", "server").
    pub name: String,
    /// Field data type (e.g. "text", "number").
    #[serde(rename = "type")]
    pub field_type: String,
}

/// User information schema containing required order form fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserInformation {
    /// Form fields required when creating an order for this game.
    #[serde(default)]
    pub forms: Vec<FormField>,
}

/// Information about a game available on DITUSI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Display name of the game (e.g. "Mobile Legends").
    pub title: String,
    /// Unique identifier / code for the game (e.g. "JSX-J01").
    pub game_code: String,
    /// Total number of available products for this game.
    pub product_amount: Option<i64>,
    /// User information and form fields required for ordering.
    #[serde(default)]
    pub user_information: Option<UserInformation>,
}

/// Query parameters for listing games via `/game`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListGamesQuery {
    /// Optional game code to filter a specific game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_code: Option<String>,
}

impl ListGamesQuery {
    /// Creates a query filtered by a specific game code.
    pub fn for_game(game_code: impl Into<String>) -> Self {
        Self {
            game_code: Some(game_code.into()),
        }
    }
}

/// Response returned by the `/game` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListGamesResponse {
    /// Status code (20 = success).
    pub status_code: i32,
    /// Status message (e.g. "SUCCESS").
    pub message: String,
    /// List of games retrieved.
    pub data: Vec<Game>,
    /// Response metadata (count, timestamp, filters).
    #[serde(default)]
    pub metadata: Option<Metadata>,
}
