use ditusi_sdk_rust::models::transaction::TransactionRequest;
use ditusi_sdk_rust::{DitusiClient, Environment, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client_id = std::env::var("DITUSI_CLIENT_ID").unwrap_or_else(|_| "421341515".to_string());
    let client_key =
        std::env::var("DITUSI_CLIENT_KEY").unwrap_or_else(|_| "0spOZMo16aFQLKwKz".to_string());

    let client = DitusiClient::new(client_id, client_key, Environment::Development)?;

    let unique_ref = format!("ORDER-{}", chrono::Utc::now().timestamp_millis());
    let form_details = serde_json::json!({
        "user_id": "123456789",
        "server": "Asia"
    });

    println!("Creating order with reference: {}", unique_ref);

    // Build the order request
    let order_req = TransactionRequest::builder("BTK60-S46", &unique_ref, form_details)
        .amount(1)
        .initial_price(1500)
        // If Hoyoverse game (Genshin Impact, Zenless Zone Zero), endUserIpAddress is mandatory:
        // .end_user_ip_address("192.168.1.1")
        .build();

    match client.create_order(order_req).await {
        Ok(order_res) => {
            println!("Order created successfully!");
            println!("Status Code: {}", order_res.status_code);
            println!("Transaction ID: {:?}", order_res.transaction_id);
            println!("Transaction Status: {:?}", order_res.status_transaction);

            if let Some(tx_id) = &order_res.transaction_id {
                println!("\nChecking order status for transaction ID: {}...", tx_id);
                let check_res = client.check_order(tx_id).await?;
                println!("Checked Status: {:?}", check_res.data.status);
                println!("Gross Amount: {:?}", check_res.data.gross_amount);
            }
        }
        Err(e) => {
            eprintln!("Failed to create order: {}", e);
        }
    }

    Ok(())
}
