//! # DITUSI SDK for Rust
//!
//! An asynchronous, type-safe Rust SDK for integrating with the [DITUSI Reseller API (v1.3.0)](https://ditusi.co.id).
//!
//! ## Key Capabilities
//! - **Product Catalog Synchronization**: List all available games and products (`/game`, `/product`).
//! - **Order Management**: Create orders with price protection and track status (`/transaction`, `/transaction/{id}`).
//! - **Reseller Balance**: Check live deposit balance and reseller tier (`/balance`).
//! - **Automated Token Management**: Automatic lazy fetching, thread-safe caching, and proactive renewal of Bearer tokens.
//! - **Precise Signatures**: Fully compliant SHA-256 request signature engine for all endpoints.
//! - **Webhook Utilities**: Parse and securely verify DITUSI webhook callbacks.
//!
//! ## Quickstart
//!
//! ```no_run
//! use ditusi_sdk_rust::{DitusiClient, Environment, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Initialize the client with your credentials
//!     let client = DitusiClient::new(
//!         "YOUR_CLIENT_ID",
//!         "YOUR_CLIENT_KEY",
//!         Environment::Development,
//!     )?;
//!
//!     // Check reseller deposit balance
//!     let balance = client.check_balance().await?;
//!     println!("Current balance: IDR {}", balance.data.deposit_balance);
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod webhook;

// Re-exports
pub use client::DitusiClient;
pub use config::{ClientConfig, ClientConfigBuilder, Environment};
pub use error::{DitusiError, Result};
