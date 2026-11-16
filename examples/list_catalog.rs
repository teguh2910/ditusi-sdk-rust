use ditusi_sdk_rust::models::product::ListProductsQuery;
use ditusi_sdk_rust::{DitusiClient, Environment, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client_id = std::env::var("DITUSI_CLIENT_ID").unwrap_or_else(|_| "421341515".to_string());
    let client_key =
        std::env::var("DITUSI_CLIENT_KEY").unwrap_or_else(|_| "0spOZMo16aFQLKwKz".to_string());

    let client = DitusiClient::new(client_id, client_key, Environment::Development)?;

    println!("Fetching available games...");
    let games_resp = client.list_games(None).await?;
    println!("Found {} games", games_resp.data.len());

    for game in &games_resp.data {
        println!(
            "- {} (code: {}, products: {})",
            game.title,
            game.game_code,
            game.product_amount.unwrap_or(0)
        );
        if let Some(user_info) = &game.user_information {
            println!("  Required fields for orders:");
            for field in &user_info.forms {
                println!("    * {} ({})", field.name, field.field_type);
            }
        }
    }

    // Fetch products for a specific game (e.g. Mobile Legends)
    if let Some(first_game) = games_resp.data.first() {
        println!("\nFetching products for game: {}", first_game.title);
        let products_resp = client
            .list_products(ListProductsQuery::by_game(&first_game.game_code))
            .await?;

        for prod in &products_resp.data {
            println!(
                "  * [{}] {} - IDR {} ({})",
                prod.code, prod.name, prod.price, prod.status
            );
        }
    }

    Ok(())
}
