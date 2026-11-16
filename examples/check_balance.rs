use ditusi_sdk_rust::{DitusiClient, Environment, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Sandbox credentials from DITUSI API Documentation
    let client_id = std::env::var("DITUSI_CLIENT_ID").unwrap_or_else(|_| "421341515".to_string());
    let client_key =
        std::env::var("DITUSI_CLIENT_KEY").unwrap_or_else(|_| "0spOZMo16aFQLKwKz".to_string());

    let client = DitusiClient::new(client_id, client_key, Environment::Development)?;

    println!("Checking balance on DITUSI Development environment...");
    let balance = client.check_balance().await?;

    println!("Status Code: {}", balance.status_code);
    println!("Partner ID: {}", balance.data.partner_id);
    println!("Reseller Tier: {}", balance.data.partner_type);
    println!("Deposit Balance: IDR {}", balance.data.deposit_balance);

    Ok(())
}
