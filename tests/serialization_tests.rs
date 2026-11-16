use ditusi_sdk_rust::models::auth::AccessTokenResponse;
use ditusi_sdk_rust::models::balance::BalanceResponse;
use ditusi_sdk_rust::models::common::ErrorResponse;
use ditusi_sdk_rust::models::game::{ListGamesQuery, ListGamesResponse};
use ditusi_sdk_rust::models::product::{ListProductsQuery, ListProductsResponse, ProductStatus};
use ditusi_sdk_rust::models::transaction::{
    CheckTransactionResponse, CreateOrderResponse, TransactionRequest, TransactionStatus,
};
use ditusi_sdk_rust::webhook::{parse_webhook_event, verify_webhook_token, WebhookEvent};

#[test]
fn test_deserialize_access_token_response() {
    let raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
        "tokenType": "Bearer",
        "expiryTime": 600
    }"#;

    let res: AccessTokenResponse = serde_json::from_str(raw).expect("deserializes access token");
    assert_eq!(res.status_code, 20);
    assert_eq!(res.message, "SUCCESS");
    assert_eq!(res.access_token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
    assert_eq!(res.token_type.as_deref(), Some("Bearer"));
    assert_eq!(res.expiry_time, Some(600));
}

#[test]
fn test_deserialize_list_games_response() {
    let raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": [
            {
                "title": "Mobile Legends",
                "gameCode": "JSX-J01",
                "productAmount": 70,
                "userInformation": {
                    "forms": [
                        {
                            "name": "user_id",
                            "type": "text"
                        },
                        {
                            "name": "server",
                            "type": "text"
                        }
                    ]
                }
            }
        ],
        "metadata": {
            "timestamp": "2024-01-01T00:00:00+07:00",
            "count": 1,
            "filters": {
                "gameCode": "JSX-J01"
            }
        }
    }"#;

    let res: ListGamesResponse = serde_json::from_str(raw).expect("deserializes games");
    assert_eq!(res.status_code, 20);
    assert_eq!(res.data.len(), 1);
    let game = &res.data[0];
    assert_eq!(game.title, "Mobile Legends");
    assert_eq!(game.game_code, "JSX-J01");
    assert_eq!(game.product_amount, Some(70));
    let forms = &game.user_information.as_ref().unwrap().forms;
    assert_eq!(forms.len(), 2);
    assert_eq!(forms[0].name, "user_id");
}

#[test]
fn test_deserialize_list_products_response() {
    let raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": [
            {
                "game_name": "Mobile Legends",
                "code": "ML5_0-S1",
                "name": "5 Diamonds ( 5 + 0 Bonus )",
                "description": "S95 - 5 Diamonds",
                "price": 1500,
                "status": "available",
                "region": "ID"
            },
            {
                "game_name": "Mobile Legends",
                "code": "ML60_0-S2",
                "name": "60 Diamonds ( 60 + 0 Bonus )",
                "description": "S95 - 60 Diamonds",
                "price": 15000,
                "status": "out of stock",
                "region": "ID"
            }
        ],
        "metadata": {
            "timestamp": "2024-01-01T00:00:00+07:00",
            "count": 2
        }
    }"#;

    let res: ListProductsResponse = serde_json::from_str(raw).expect("deserializes products");
    assert_eq!(res.status_code, 20);
    assert_eq!(res.data.len(), 2);
    assert_eq!(res.data[0].status, ProductStatus::Available);
    assert_eq!(res.data[1].status, ProductStatus::OutOfStock);
}

#[test]
fn test_deserialize_create_order_response() {
    let raw = r#"{
        "statusCode": 21,
        "message": "SUCCESS",
        "statusTransaction": "PENDING ORDER",
        "transactionId": "D2C-30330245313890503",
        "originalTransactionReferenceId": "ORDER-12345-ABC"
    }"#;

    let res: CreateOrderResponse = serde_json::from_str(raw).expect("deserializes create order");
    assert_eq!(res.status_code, 21);
    assert_eq!(res.status_transaction, Some(TransactionStatus::PendingOrder));
    assert_eq!(res.transaction_id.as_deref(), Some("D2C-30330245313890503"));
    assert_eq!(
        res.original_transaction_reference_id.as_deref(),
        Some("ORDER-12345-ABC")
    );
}

#[test]
fn test_deserialize_check_transaction_response() {
    let raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": {
            "status": "SUCCESS",
            "transactionId": "D2C-30330245313890503",
            "transactionReferenceId": "ORDER-12345-ABC",
            "gross_amount": 1560.0,
            "form_details": {
                "user_id": "123456789",
                "server": "Asia"
            },
            "product": {
                "code": "ML5_0-S1",
                "name": "5 Diamonds ( 5 + 0 Bonus )",
                "amount": 1
            }
        },
        "metadata": {
            "timestamp": "2024-01-01T00:00:00+07:00"
        }
    }"#;

    let res: CheckTransactionResponse =
        serde_json::from_str(raw).expect("deserializes check transaction");
    assert_eq!(res.status_code, 20);
    assert_eq!(res.data.status, TransactionStatus::Success);
    assert_eq!(res.data.transaction_id, "D2C-30330245313890503");
    assert_eq!(
        res.data.transaction_reference_id.as_deref(),
        Some("ORDER-12345-ABC")
    );
    assert_eq!(res.data.gross_amount, Some(1560.0));
    assert!(res.data.product.is_some());
}

#[test]
fn test_deserialize_balance_response() {
    let raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": {
            "partnerId": 123456,
            "partnerType": "RESELLER GOLD",
            "DepositBalance": 100000
        },
        "metadata": {
            "timestamp": "2024-01-01T00:00:00+07:00"
        }
    }"#;

    let res: BalanceResponse = serde_json::from_str(raw).expect("deserializes balance");
    assert_eq!(res.status_code, 20);
    assert_eq!(res.data.partner_id, 123456);
    assert_eq!(res.data.partner_type, "RESELLER GOLD");
    assert_eq!(res.data.deposit_balance, 100000);
}

#[test]
fn test_deserialize_error_response() {
    let raw = r#"{
        "statusCode": 40,
        "message": "VALIDATION_FAILED"
    }"#;

    let res: ErrorResponse = serde_json::from_str(raw).expect("deserializes error");
    assert_eq!(res.status_code, 40);
    assert_eq!(res.message, "VALIDATION_FAILED");
}

#[test]
fn test_transaction_request_serialization() {
    let form = serde_json::json!({
        "user_id": "123456789",
        "server": "Asia"
    });

    let req = TransactionRequest::builder("ML5_0-S1", "ORDER-12345-ABC", form)
        .amount(2)
        .initial_price(3000)
        .end_user_ip_address("192.168.1.1")
        .build();

    let json = serde_json::to_string(&req).expect("serializes request");
    assert!(json.contains(r#""productCode":"ML5_0-S1""#));
    assert!(json.contains(r#""amount":2"#));
    assert!(json.contains(r#""transactionReferenceId":"ORDER-12345-ABC""#));
    assert!(json.contains(r#""initialPrice":3000"#));
    assert!(json.contains(r#""endUserIpAddress":"192.168.1.1""#));
}

#[test]
fn test_query_compact_serialization() {
    let query_empty = ListGamesQuery::default();
    assert_eq!(serde_json::to_string(&query_empty).unwrap(), "{}");

    let query_game = ListGamesQuery::for_game("JSX-J01");
    assert_eq!(
        serde_json::to_string(&query_game).unwrap(),
        r#"{"gameCode":"JSX-J01"}"#
    );

    let query_product = ListProductsQuery::by_product("ML5_0-S1");
    assert_eq!(
        serde_json::to_string(&query_product).unwrap(),
        r#"{"productCode":"ML5_0-S1"}"#
    );
}

#[test]
fn test_webhook_parsing_and_verification() {
    // 1. Verification
    assert!(verify_webhook_token("secret_token_123", "secret_token_123"));
    assert!(verify_webhook_token(
        "Bearer secret_token_123",
        "secret_token_123"
    ));
    assert!(!verify_webhook_token(
        "Bearer wrong_token",
        "secret_token_123"
    ));

    // 2. Transaction Webhook Parsing
    let tx_raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": {
            "status": "SUCCESS",
            "transactionId": "D2A-48354327428765262",
            "transactionReferenceId": "FDBSLHK4K53433",
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

    match parse_webhook_event(tx_raw).expect("parses transaction webhook") {
        WebhookEvent::Transaction(payload) => {
            assert_eq!(payload.data.status, TransactionStatus::Success);
            assert_eq!(payload.data.transaction_id, "D2A-48354327428765262");
            assert_eq!(
                payload.data.transaction_reference_id.as_deref(),
                Some("FDBSLHK4K53433")
            );
        }
        _ => panic!("Expected Transaction webhook event"),
    }

    // 3. Product Webhook Parsing
    let prod_raw = r#"{
        "statusCode": 20,
        "message": "SUCCESS",
        "data": {
            "gameName": "Mobile Legends",
            "productCode": "ML5_0-S1",
            "productName": "5 Diamonds ( 5 + 0 Bonus )",
            "description": "S95 - 5 Diamonds",
            "price": 1500,
            "inGameCurrency": "Diamonds",
            "status": "available"
        },
        "metadata": {
            "timestamp": "2025-01-01T00:00:00+07:00"
        }
    }"#;

    match parse_webhook_event(prod_raw).expect("parses product webhook") {
        WebhookEvent::Product(payload) => {
            assert_eq!(payload.data.product_code, "ML5_0-S1");
            assert_eq!(payload.data.price, 1500);
            assert_eq!(payload.data.status, "available");
        }
        _ => panic!("Expected Product webhook event"),
    }
}
