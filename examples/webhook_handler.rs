use ditusi_sdk_rust::webhook::{parse_webhook_event, verify_webhook_token, WebhookEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configured_secret = "my_reseller_webhook_secret_token";

    // Simulated incoming HTTP headers and payload
    let incoming_auth_header = "Bearer my_reseller_webhook_secret_token";
    let incoming_body = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": {
            "status": "SUCCESS",
            "transactionId": "D2A-48354327428765262",
            "transactionReferenceId": "ORDER-12345-ABC",
            "errorMessage": "",
            "product": {
                "code": "ML5_0-S1",
                "name": "5 Diamonds ( 5 + 0 Bonus )",
                "amount": 1
            }
        },
        "metadata": {
            "timestamp": "2025-01-01T00:00:00+07:00"
        }
    }"#;

    // 1. Verify webhook authorization header
    if !verify_webhook_token(incoming_auth_header, configured_secret) {
        eprintln!("Rejected unauthenticated webhook request!");
        return Ok(());
    }
    println!("Webhook verification passed!");

    // 2. Parse and handle the event
    match parse_webhook_event(incoming_body)? {
        WebhookEvent::Transaction(tx) => {
            println!("Received Order Status Update:");
            println!("  Transaction ID: {}", tx.data.transaction_id);
            println!("  Reference ID: {:?}", tx.data.transaction_reference_id);
            println!("  New Status: {}", tx.data.status);
            if let Some(prod) = tx.data.product {
                println!("  Product: {} ({}) x{}", prod.name, prod.code, prod.amount);
            }
        }
        WebhookEvent::Product(prod) => {
            println!("Received Product Catalog Change:");
            println!("  Product Code: {}", prod.data.product_code);
            println!("  Name: {}", prod.data.product_name);
            println!("  Price: IDR {}", prod.data.price);
            println!("  Availability: {}", prod.data.status);
        }
        WebhookEvent::Unknown(val) => {
            println!("Received unrecognized event: {}", val);
        }
    }

    Ok(())
}
