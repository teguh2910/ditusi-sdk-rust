use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, warn};

use crate::auth::{
    calculate_api_signature, calculate_token_signature, current_timestamp, TokenManager,
};
use crate::config::{ClientConfig, ClientConfigBuilder, Environment};
use crate::error::{DitusiError, Result};
use crate::models::auth::AccessTokenResponse;
use crate::models::balance::BalanceResponse;
use crate::models::common::{ErrorResponse, SandboxWebhookTestRequest, SandboxWebhookTestResponse};
use crate::models::game::{ListGamesQuery, ListGamesResponse};
use crate::models::product::{ListProductsQuery, ListProductsResponse};
use crate::models::transaction::{
    CheckTransactionResponse, CreateOrderResponse, TransactionRequest,
};

/// High-level client for interacting with the DITUSI Reseller API.
#[derive(Debug, Clone)]
pub struct DitusiClient {
    config: ClientConfig,
    http: Client,
    token_manager: TokenManager,
}

impl DitusiClient {
    /// Returns a new [`ClientConfigBuilder`] for configuring a client.
    pub fn builder() -> ClientConfigBuilder {
        ClientConfig::builder()
    }

    /// Creates a new [`DitusiClient`] with basic credentials and environment.
    pub fn new(
        client_id: impl Into<String>,
        client_key: impl Into<String>,
        environment: Environment,
    ) -> Result<Self> {
        let config = ClientConfig::builder()
            .client_id(client_id)
            .client_key(client_key)
            .environment(environment)
            .build()?;
        Self::from_config(config)
    }

    /// Creates a new [`DitusiClient`] from a [`ClientConfig`].
    pub fn from_config(config: ClientConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(DitusiError::Http)?;

        let token_manager = if let Some(ref static_tok) = config.static_token {
            TokenManager::with_static_token(static_tok)
        } else {
            TokenManager::new()
        };

        Ok(Self {
            config,
            http,
            token_manager,
        })
    }

    /// Returns a reference to the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Explicitly generates a new Bearer access token via `GET /access-token`.
    ///
    /// The token is valid for 10 minutes (600 seconds) and automatically updates the internal cache.
    pub async fn get_access_token(&self) -> Result<AccessTokenResponse> {
        let timestamp = current_timestamp();
        let signature = calculate_token_signature(&self.config.client_key, &timestamp);
        let url = self.config.environment.access_token_url();

        debug!(target: "ditusi_sdk", "Requesting access token from {}", url);

        let response = self
            .http
            .get(&url)
            .header("X-CLIENT-ID", &self.config.client_id)
            .header("X-TIMESTAMP", &timestamp)
            .header("X-SIGNATURE", &signature)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        let body_text = response.text().await?;

        if !status.is_success() {
            if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&body_text) {
                return Err(DitusiError::api(
                    err_resp.status_code,
                    err_resp.message,
                    Some(body_text),
                ));
            }
            return Err(DitusiError::api(
                status.as_u16() as i32,
                format!("Failed to retrieve access token (HTTP {})", status),
                Some(body_text),
            ));
        }

        let token_resp: AccessTokenResponse = serde_json::from_str(&body_text)?;
        if token_resp.status_code != 20 {
            return Err(DitusiError::api(
                token_resp.status_code,
                token_resp.message,
                Some(body_text),
            ));
        }

        self.token_manager.update_from_response(&token_resp).await?;
        Ok(token_resp)
    }

    /// Internal method to ensure a valid Bearer token is available, fetching one if necessary.
    async fn ensure_bearer_token(&self) -> Result<String> {
        if let Some(cached) = self.token_manager.get_cached_token().await {
            return Ok(cached);
        }

        if !self.config.auto_refresh_token {
            return Err(DitusiError::Authentication(
                "No valid Bearer token cached and auto-refresh is disabled.".into(),
            ));
        }

        let resp = self.get_access_token().await?;
        Ok(resp.access_token)
    }

    /// Internal helper to execute a signed API request and handle errors / auto-retry on 401.
    async fn execute_signed_request<T, Q, B>(
        &self,
        method: Method,
        endpoint_path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        match self
            .send_signed_request_attempt(&method, endpoint_path, query, body)
            .await
        {
            Ok(result) => Ok(result),
            Err(DitusiError::Api { status_code, .. }) if (status_code == 401 || status_code == 45) && self.config.auto_refresh_token => {
                warn!(target: "ditusi_sdk", "Received 401 / 45 unauthorized error. Invalidating token cache and retrying...");
                self.token_manager.invalidate().await;
                self.send_signed_request_attempt(&method, endpoint_path, query, body).await
            }
            Err(e) => Err(e),
        }
    }

    /// Executes a single signed request attempt.
    async fn send_signed_request_attempt<T, Q, B>(
        &self,
        method: &Method,
        endpoint_path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let bearer_token = self.ensure_bearer_token().await?;
        let timestamp = current_timestamp();

        // Calculate full path for signature (e.g. /api/dev/v1/game)
        let prefix = self.config.environment.path_prefix();
        let full_signature_path = format!("{}{}", prefix, endpoint_path);

        // Serialize compact body representation for signature
        let compact_json_body = match (method, query, body) {
            (&Method::GET, Some(q), _) => serde_json::to_string(q)?,
            (&Method::GET, None, _) => "{}".to_string(),
            (_, _, Some(b)) => serde_json::to_string(b)?,
            _ => "{}".to_string(),
        };

        let signature = calculate_api_signature(
            &full_signature_path,
            &self.config.client_key,
            &timestamp,
            &compact_json_body,
        );

        let base = self.config.environment.base_url();
        let url = format!("{}{}", base, endpoint_path);

        let mut req = self
            .http
            .request(method.clone(), &url)
            .header("X-CLIENT-ID", &self.config.client_id)
            .header("X-TIMESTAMP", &timestamp)
            .header("X-SIGNATURE", &signature)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .header("Content-Type", "application/json");

        if method == Method::GET {
            if let Some(q) = query {
                req = req.query(q);
            }
        } else if body.is_some() {
            req = req.body(compact_json_body);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body_text) {
                return Err(DitusiError::api(err.status_code, err.message, Some(body_text)));
            }
            return Err(DitusiError::api(
                status.as_u16() as i32,
                format!("Request failed with HTTP status {}", status),
                Some(body_text),
            ));
        }

        // Check for application-level error codes in JSON
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Some(code) = val.get("statusCode").and_then(|c| c.as_i64()) {
                // 20 = Success, 21 = Created (Pending Order)
                if code != 20 && code != 21 {
                    let msg = val
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown API Error")
                        .to_string();
                    return Err(DitusiError::api(code as i32, msg, Some(body_text)));
                }
            }
        }

        let parsed: T = serde_json::from_str(&body_text)?;
        Ok(parsed)
    }

    /// Retrieves the list of available games (`GET /game`).
    ///
    /// Pass `Some(ListGamesQuery::for_game("..."))` to filter for a specific game code.
    pub async fn list_games(&self, query: Option<ListGamesQuery>) -> Result<ListGamesResponse> {
        self.execute_signed_request(
            Method::GET,
            "/game",
            query.as_ref(),
            None::<&serde_json::Value>,
        )
        .await
    }

    /// Retrieves available products (`GET /product`).
    ///
    /// Requires either `game_code` or `product_code` in the query.
    pub async fn list_products(&self, query: ListProductsQuery) -> Result<ListProductsResponse> {
        self.execute_signed_request(
            Method::GET,
            "/product",
            Some(&query),
            None::<&serde_json::Value>,
        )
        .await
    }

    /// Places a new top-up order (`POST /transaction`).
    ///
    /// Upon successful creation, order cost is immediately deducted from your reseller balance.
    pub async fn create_order(&self, req: TransactionRequest) -> Result<CreateOrderResponse> {
        self.execute_signed_request(
            Method::POST,
            "/transaction",
            None::<&serde_json::Value>,
            Some(&req),
        )
        .await
    }

    /// Checks the status of an order (`GET /transaction/{transactionId}`).
    ///
    /// `transaction_id_or_ref` can be either the DITUSI transaction ID (e.g. `D2C-30330245313890503`)
    /// or your original `transactionReferenceId`.
    pub async fn check_order(&self, transaction_id_or_ref: &str) -> Result<CheckTransactionResponse> {
        let path = format!("/transaction/{}", transaction_id_or_ref);
        self.execute_signed_request(
            Method::GET,
            &path,
            None::<&serde_json::Value>,
            None::<&serde_json::Value>,
        )
        .await
    }

    /// Retrieves current reseller account balance and tier (`GET /balance`).
    pub async fn check_balance(&self) -> Result<BalanceResponse> {
        self.execute_signed_request(
            Method::GET,
            "/balance",
            None::<&serde_json::Value>,
            None::<&serde_json::Value>,
        )
        .await
    }

    /// Sends a test transaction webhook callback in the sandbox environment (`POST /callback/transaction`).
    pub async fn test_transaction_webhook(
        &self,
        destination_url: impl Into<String>,
        transaction_id: impl Into<String>,
    ) -> Result<SandboxWebhookTestResponse> {
        let req = SandboxWebhookTestRequest {
            url: destination_url.into(),
            transaction_id: Some(transaction_id.into()),
            product_code: None,
        };

        let base = self.config.environment.base_url();
        let url = format!("{}/callback/transaction", base);

        let resp = self
            .http
            .post(&url)
            .header("X-CLIENT-ID", &self.config.client_id)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(DitusiError::api(
                status.as_u16() as i32,
                format!("Sandbox webhook test failed with HTTP {}", status),
                Some(text),
            ));
        }

        let test_resp: SandboxWebhookTestResponse = serde_json::from_str(&text)?;
        Ok(test_resp)
    }

    /// Sends a test product webhook callback in the sandbox environment (`POST /callback/product`).
    pub async fn test_product_webhook(
        &self,
        destination_url: impl Into<String>,
        product_code: impl Into<String>,
    ) -> Result<SandboxWebhookTestResponse> {
        let req = SandboxWebhookTestRequest {
            url: destination_url.into(),
            transaction_id: None,
            product_code: Some(product_code.into()),
        };

        let base = self.config.environment.base_url();
        let url = format!("{}/callback/product", base);

        let resp = self
            .http
            .post(&url)
            .header("X-CLIENT-ID", &self.config.client_id)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(DitusiError::api(
                status.as_u16() as i32,
                format!("Sandbox product webhook test failed with HTTP {}", status),
                Some(text),
            ));
        }

        let test_resp: SandboxWebhookTestResponse = serde_json::from_str(&text)?;
        Ok(test_resp)
    }
}
