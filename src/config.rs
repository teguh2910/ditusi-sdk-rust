use std::time::Duration;
use crate::error::{DitusiError, Result};

/// API Environment for DITUSI.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Environment {
    /// Development / Sandbox environment: `https://api.ditusi.co.id/api/dev/v1`
    #[default]
    Development,
    /// Production environment: `https://api.ditusi.co.id/api/v1`
    Production,
    /// Custom base URL (useful for mock testing or reverse proxies)
    Custom(String),
}

impl Environment {
    /// Returns the base URL for standard API endpoints.
    pub fn base_url(&self) -> &str {
        match self {
            Self::Development => "https://api.ditusi.co.id/api/dev/v1",
            Self::Production => "https://api.ditusi.co.id/api/v1",
            Self::Custom(url) => url.trim_end_matches('/'),
        }
    }

    /// Returns the path prefix used in signature calculation (e.g. `/api/dev/v1` or `/api/v1`).
    pub fn path_prefix(&self) -> String {
        match self {
            Self::Development => "/api/dev/v1".to_string(),
            Self::Production => "/api/v1".to_string(),
            Self::Custom(url) => {
                // Extract path component from custom URL, or default to ""
                if let Ok(parsed) = reqwest::Url::parse(url) {
                    let path = parsed.path().trim_end_matches('/');
                    if !path.is_empty() {
                        return path.to_string();
                    }
                }
                "".to_string()
            }
        }
    }

    /// Returns the URL for the `/access-token` endpoint.
    ///
    /// Per DITUSI docs: "For authentication endpoint, staging and production use the same url (/api/v1/access-token)."
    pub fn access_token_url(&self) -> String {
        match self {
            Self::Development | Self::Production => {
                "https://api.ditusi.co.id/api/v1/access-token".to_string()
            }
            Self::Custom(base) => {
                format!("{}/access-token", base.trim_end_matches('/'))
            }
        }
    }

    /// Returns the path used for the `/access-token` endpoint in signature calculation.
    pub fn access_token_path(&self) -> &str {
        match self {
            Self::Development | Self::Production => "/api/v1/access-token",
            Self::Custom(_) => "/access-token",
        }
    }
}

/// Configuration settings for [`crate::DitusiClient`].
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Your unique Client ID from DITUSI API Management.
    pub client_id: String,
    /// Your secret Client Key (secret) from DITUSI API Management.
    pub client_key: String,
    /// The target environment (Development or Production).
    pub environment: Environment,
    /// Request timeout for HTTP operations.
    pub timeout: Duration,
    /// Whether the SDK should automatically request and refresh Bearer tokens.
    pub auto_refresh_token: bool,
    /// Optional static Bearer token override (bypasses automatic fetching).
    pub static_token: Option<String>,
}

impl ClientConfig {
    /// Returns a new [`ClientConfigBuilder`] for configuring the client.
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }
}

/// Builder for constructing a [`ClientConfig`].
#[derive(Debug, Clone, Default)]
pub struct ClientConfigBuilder {
    client_id: Option<String>,
    client_key: Option<String>,
    environment: Environment,
    timeout: Option<Duration>,
    auto_refresh_token: Option<bool>,
    static_token: Option<String>,
}

impl ClientConfigBuilder {
    /// Sets the Client ID.
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Sets the Client Key (secret).
    pub fn client_key(mut self, client_key: impl Into<String>) -> Self {
        self.client_key = Some(client_key.into());
        self
    }

    /// Sets the target environment (e.g. `Environment::Development` or `Environment::Production`).
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Sets the HTTP request timeout (defaults to 30 seconds).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Configures whether to automatically manage access tokens (defaults to `true`).
    pub fn auto_refresh_token(mut self, auto_refresh: bool) -> Self {
        self.auto_refresh_token = Some(auto_refresh);
        self
    }

    /// Sets a manual/static Bearer token, bypassing automatic token generation.
    pub fn static_token(mut self, token: impl Into<String>) -> Self {
        self.static_token = Some(token.into());
        self
    }

    /// Builds the [`ClientConfig`].
    pub fn build(self) -> Result<ClientConfig> {
        let client_id = self
            .client_id
            .ok_or_else(|| DitusiError::MissingRequiredField("client_id is required".into()))?;
        let client_key = self
            .client_key
            .ok_or_else(|| DitusiError::MissingRequiredField("client_key is required".into()))?;

        Ok(ClientConfig {
            client_id,
            client_key,
            environment: self.environment,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            auto_refresh_token: self.auto_refresh_token.unwrap_or(true),
            static_token: self.static_token,
        })
    }
}
