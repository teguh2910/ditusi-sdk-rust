use ditusi_sdk_rust::auth::{
    calculate_api_signature, calculate_token_signature, sha256_hex,
};

#[test]
fn test_access_token_signature() {
    let client_key = "YOUR_CLIENT_KEY";
    let timestamp = "2024-01-01T00:00:00+07:00";

    let signature = calculate_token_signature(client_key, timestamp);

    // sha256("YOUR_CLIENT_KEY:2024-01-01T00:00:00+07:00")
    let expected = sha256_hex("YOUR_CLIENT_KEY:2024-01-01T00:00:00+07:00");
    assert_eq!(signature, expected);
    assert_eq!(signature.len(), 64);
}

#[test]
fn test_api_post_signature_matches_docs() {
    let path = "/api/dev/v1/transaction";
    let client_key = "YOUR_CLIENT_KEY";
    let timestamp = "2024-01-01T00:00:00+07:00";
    let compact_body = r#"{"productCode":"ML5_0-S1","amount":1}"#;

    let signature = calculate_api_signature(path, client_key, timestamp, compact_body);

    let expected = sha256_hex(&format!(
        "{}:{}:{}:{}",
        path, client_key, timestamp, compact_body
    ));
    assert_eq!(signature, expected);
}

#[test]
fn test_api_get_empty_query_signature() {
    let path = "/api/dev/v1/game";
    let client_key = "0spOZMo16aFQLKwKz";
    let timestamp = "2024-01-01T00:00:00+07:00";
    let empty_body = "{}";

    let signature = calculate_api_signature(path, client_key, timestamp, empty_body);

    let raw = "/api/dev/v1/game:0spOZMo16aFQLKwKz:2024-01-01T00:00:00+07:00:{}";
    let expected = sha256_hex(raw);
    assert_eq!(signature, expected);
}

#[test]
fn test_api_get_with_query_signature() {
    let path = "/api/dev/v1/product";
    let client_key = "0spOZMo16aFQLKwKz";
    let timestamp = "2024-01-01T00:00:00+07:00";
    let query_body = r#"{"productCode":"ML5_0-S1"}"#;

    let signature = calculate_api_signature(path, client_key, timestamp, query_body);

    let raw = "/api/dev/v1/product:0spOZMo16aFQLKwKz:2024-01-01T00:00:00+07:00:{\"productCode\":\"ML5_0-S1\"}";
    let expected = sha256_hex(raw);
    assert_eq!(signature, expected);
}
