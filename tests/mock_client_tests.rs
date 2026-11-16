use ditusi_sdk_rust::config::{ClientConfig, Environment};
use ditusi_sdk_rust::models::transaction::TransactionRequest;
use ditusi_sdk_rust::DitusiClient;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_auto_token_and_balance_flow() {
    let mock_server = MockServer::start().await;

    // 1. Mock the /access-token endpoint
    let token_body = serde_json::json!({
        "statusCode": 20,
        "message": "SUCCESS",
        "accessToken": "mocked_jwt_token_12345",
        "tokenType": "Bearer",
        "expiryTime": 600
    });

    Mock::given(method("GET"))
        .and(path("/access-token"))
        .and(header_exists("X-CLIENT-ID"))
        .and(header_exists("X-TIMESTAMP"))
        .and(header_exists("X-SIGNATURE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&token_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    // 2. Mock the /balance endpoint
    let balance_body = serde_json::json!({
        "statusCode": 20,
        "message": "SUCCESS",
        "data": {
            "partnerId": 421341515,
            "partnerType": "RESELLER GOLD",
            "DepositBalance": 250000
        },
        "metadata": {
            "timestamp": "2024-01-01T00:00:00+07:00"
        }
    });

    Mock::given(method("GET"))
        .and(path("/balance"))
        .and(header_exists("X-CLIENT-ID"))
        .and(header_exists("X-TIMESTAMP"))
        .and(header_exists("X-SIGNATURE"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&balance_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = ClientConfig::builder()
        .client_id("421341515")
        .client_key("0spOZMo16aFQLKwKz")
        .environment(Environment::Custom(mock_server.uri()))
        .build()
        .expect("builds client config");

    let client = DitusiClient::from_config(config).expect("creates client");

    // Call check_balance - this should automatically call /access-token first, cache it, then call /balance
    let balance = client.check_balance().await.expect("fetches balance");

    assert_eq!(balance.status_code, 20);
    assert_eq!(balance.data.partner_id, 421341515);
    assert_eq!(balance.data.deposit_balance, 250000);
}

#[tokio::test]
async fn test_create_order_flow() {
    let mock_server = MockServer::start().await;

    // Static token config to test static_token mode
    let config = ClientConfig::builder()
        .client_id("421341515")
        .client_key("0spOZMo16aFQLKwKz")
        .environment(Environment::Custom(mock_server.uri()))
        .static_token("pre_generated_bearer_token")
        .build()
        .expect("builds config");

    let client = DitusiClient::from_config(config).expect("creates client");

    let order_resp = serde_json::json!({
        "statusCode": 21,
        "message": "SUCCESS",
        "statusTransaction": "PENDING ORDER",
        "transactionId": "D2C-30330245313890503",
        "originalTransactionReferenceId": "ORDER-999"
    });

    Mock::given(method("POST"))
        .and(path("/transaction"))
        .and(header_exists("X-CLIENT-ID"))
        .and(header_exists("X-TIMESTAMP"))
        .and(header_exists("X-SIGNATURE"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&order_resp))
        .expect(1)
        .mount(&mock_server)
        .await;

    let req = TransactionRequest::builder(
        "ML5_0-S1",
        "ORDER-999",
        serde_json::json!({"user_id": "123", "server": "Asia"}),
    )
    .amount(1)
    .build();

    let created = client.create_order(req).await.expect("creates order");
    assert_eq!(created.status_code, 21);
    assert_eq!(created.transaction_id.as_deref(), Some("D2C-30330245313890503"));
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    let err_body = serde_json::json!({
        "statusCode": 31,
        "message": "PRODUCT_EMPTY"
    });

    Mock::given(method("POST"))
        .and(path("/transaction"))
        .respond_with(ResponseTemplate::new(422).set_body_json(&err_body))
        .mount(&mock_server)
        .await;

    let client = DitusiClient::builder()
        .client_id("421341515")
        .client_key("0spOZMo16aFQLKwKz")
        .environment(Environment::Custom(mock_server.uri()))
        .static_token("token")
        .build()
        .map(DitusiClient::from_config)
        .unwrap()
        .unwrap();

    let req = TransactionRequest::builder(
        "OUT_OF_STOCK",
        "ORDER-101",
        serde_json::json!({"user_id": "123"}),
    )
    .build();

    let res = client.create_order(req).await;
    match res {
        Err(ditusi_sdk_rust::DitusiError::Api {
            status_code,
            message,
            ..
        }) => {
            assert_eq!(status_code, 31);
            assert_eq!(message, "PRODUCT_EMPTY");
        }
        other => panic!("Expected ApiError, got {:?}", other),
    }
}
