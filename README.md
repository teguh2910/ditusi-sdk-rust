# DITUSI SDK for Rust (`ditusi-sdk-rust`)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org)

An idiomatic, type-safe, asynchronous Rust SDK for integrating with the **[DITUSI Reseller API (v1.3.0)](https://ditusi.co.id)**.

---

## Features

- ⚡ **Async Tokio Runtime**: Built from the ground up on `tokio` and `reqwest` with `rustls`.
- 🔐 **Automated Token Lifecycle**: Handles lazy fetching, thread-safe caching, and proactive renewal of 10-minute Bearer tokens automatically.
- 🔏 **Compliant Signature Engine**: Calculates SHA-256 HMAC-style signatures for both `/access-token` requests and compact JSON GET/POST endpoints according to DITUSI specification.
- 🎮 **Catalog Synchronization**: Type-safe querying of games and products with dynamic form definitions (`/game`, `/product`).
- 🛒 **Transaction Management**: Fluent builders for order creation with price protection (`initialPrice`), duplicate prevention (`transactionReferenceId`), and mandatory IP tracking for Hoyoverse titles.
- 💰 **Balance Monitoring**: Check live deposit balance and reseller partnership tier (`/balance`).
- 🪝 **Webhook Tools**: Framework-agnostic signature verification and payload decoders for transaction and product events.
- 🧪 **Sandbox Testing Endpoints**: Built-in support for `/callback/transaction` and `/callback/product` testing.

---

## Installation

Add `ditusi-sdk-rust` and `tokio` to your `Cargo.toml`:

```toml
[dependencies]
ditusi-sdk-rust = { path = "/Users/teguhyuhono/.gemini/antigravity/scratch/ditusi-sdk-rust" }
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
```

---

## Quickstart

### 1. Initialize the Client

```rust
use ditusi_sdk_rust::{DitusiClient, Environment, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Development / Sandbox environment
    let client = DitusiClient::new(
        "421341515",          // Sandbox Client ID
        "0spOZMo16aFQLKwKz",  // Sandbox Client Key
        Environment::Development,
    )?;

    // Check balance
    let balance = client.check_balance().await?;
    println!("Reseller: {}", balance.data.partner_type);
    println!("Balance: IDR {}", balance.data.deposit_balance);

    Ok(())
}
```

Or using the fluent builder:

```rust
use std::time::Duration;
use ditusi_sdk_rust::{DitusiClient, Environment, Result};

let client = DitusiClient::builder()
    .client_id("YOUR_CLIENT_ID")
    .client_key("YOUR_CLIENT_KEY")
    .environment(Environment::Production)
    .timeout(Duration::from_secs(45))
    .auto_refresh_token(true)
    .build()
    .map(DitusiClient::from_config)?;
```

---

## Usage Guide

### Syncing Games & Products

```rust
use ditusi_sdk_rust::models::product::ListProductsQuery;

// 1. Fetch all games
let games = client.list_games(None).await?;
for game in &games.data {
    println!("Game: {} (Code: {})", game.title, game.game_code);
}

// 2. Fetch products for a specific game
let query = ListProductsQuery::by_game("JSX-J01");
let products = client.list_products(query).await?;
for product in &products.data {
    println!("Product: [{}] {} - IDR {}", product.code, product.name, product.price);
}
```

### Creating Top-up Orders

```rust
use ditusi_sdk_rust::models::transaction::TransactionRequest;

let form = serde_json::json!({
    "user_id": "123456789",
    "server": "Asia"
});

let order_req = TransactionRequest::builder("ML5_0-S1", "ORDER-12345-ABC", form)
    .amount(1)
    .initial_price(1500) // Price protection
    // Required for Hoyoverse titles (Genshin Impact, Zenless Zone Zero, etc.)
    .end_user_ip_address("192.168.1.1") 
    .build();

let response = client.create_order(order_req).await?;
println!("Transaction Created: {:?}", response.transaction_id);
println!("Status: {:?}", response.status_transaction);
```

### Checking Order Status

You can query an order using either the DITUSI transaction ID or your own reference ID:

```rust
// Query by DITUSI ID or Reseller Reference ID
let status_resp = client.check_order("D2C-30330245313890503").await?;
println!("Current Status: {}", status_resp.data.status);
println!("Gross Amount: {:?}", status_resp.data.gross_amount);
```

### Handling Webhook Callbacks

The SDK provides framework-agnostic utilities to verify and parse incoming webhook events:

```rust
use ditusi_sdk_rust::webhook::{verify_webhook_token, parse_webhook_event, WebhookEvent};

// In your Axum, Actix, or Warp handler:
fn handle_webhook(auth_header: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Verify token
    if !verify_webhook_token(auth_header, "YOUR_WEBHOOK_SECRET_TOKEN") {
        return Err("Unauthorized webhook".into());
    }

    // 2. Parse event
    match parse_webhook_event(body)? {
        WebhookEvent::Transaction(tx) => {
            println!("Order status update: {} -> {}", tx.data.transaction_id, tx.data.status);
        }
        WebhookEvent::Product(prod) => {
            println!("Product update: {} is now {}", prod.data.product_code, prod.data.status);
        }
        WebhookEvent::Unknown(val) => {
            println!("Unrecognized webhook payload: {}", val);
        }
    }

    Ok(())
}
```

### Testing Webhooks in Sandbox

Trigger test callbacks back to your local or staging server:

```rust
// Simulate a transaction webhook event
client.test_transaction_webhook("https://your-domain.com/webhook", "D2C-16902220334421").await?;

// Simulate a product catalog update event
client.test_product_webhook("https://your-domain.com/webhook", "AOV4800-S1000").await?;
```

---

## Sandbox Reference

| Setting | Value |
|---------|-------|
| **Base URL (Dev)** | `https://api.ditusi.co.id/api/dev/v1` |
| **Base URL (Prod)** | `https://api.ditusi.co.id/api/v1` |
| **Sandbox Client ID** | `421341515` |
| **Sandbox Client Key** | `0spOZMo16aFQLKwKz` |

### Test Product Codes (Development)
- Automatic Success: `BTK60-S46`
- Login-based Success: `KAFRA-VIP-8$`
- Out of Stock: `AOV230-S18`
- Insufficient Balance: `BTK300-S62`
- Hoyoverse Missing IP: `GI60-S54-BR`

---

## Running Examples & Tests

Run all unit and integration tests:

```bash
cargo test
```

Run test examples:

```bash
cargo run --example check_balance
cargo run --example list_catalog
cargo run --example create_order
cargo run --example webhook_handler
```

Generate documentation:

```bash
cargo doc --open --no-deps
```

---

## License

This project is licensed under the [MIT License](LICENSE).
