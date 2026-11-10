//! OAuth-style access-token acquisition and refresh.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::{Error, Result};

/// The default QQ Bot token endpoint.
pub const DEFAULT_TOKEN_URL: &str = crate::constants::TOKEN_URL;

/// The token type used by the QQ Bot OpenAPI.
pub const TOKEN_TYPE: &str = crate::constants::TOKEN_TYPE;

/// QQ Bot application credentials.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credentials {
    /// Application ID.
    #[serde(rename = "appId", alias = "appid")]
    pub app_id: String,
    /// Application secret.
    #[serde(rename = "clientSecret", alias = "secret")]
    pub app_secret: String,
}

impl Credentials {
    /// Creates credentials.
    #[must_use]
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        }
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Credentials")
            .field("app_id", &self.app_id)
            .field("app_secret", &"<redacted>")
            .finish()
    }
}

/// An access token returned by the token endpoint.
#[allow(clippy::struct_field_names)]
#[derive(Clone)]
pub struct AccessToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
    expires_at: SystemTime,
}

impl AccessToken {
    /// Creates an access token.
    ///
    /// This constructor is useful when implementing a custom [`TokenSource`].
    #[must_use]
    pub fn new(
        access_token: impl Into<String>,
        token_type: impl Into<String>,
        expires_in: u64,
        expires_at: SystemTime,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            token_type: token_type.into(),
            expires_in,
            expires_at,
        }
    }

    /// Returns the raw access token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.access_token
    }

    /// Returns the token type, usually `QQBot`.
    #[must_use]
    pub fn token_type(&self) -> &str {
        &self.token_type
    }

    /// Returns the lifetime supplied by the server, in seconds.
    #[must_use]
    pub const fn expires_in(&self) -> u64 {
        self.expires_in
    }

    /// Returns the absolute expiry time.
    #[must_use]
    pub const fn expires_at(&self) -> SystemTime {
        self.expires_at
    }

    /// Returns `true` when the token is still usable with a 10 second leeway.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_valid_for(Duration::from_secs(10))
    }

    /// Returns `true` when the token remains valid for `leeway`.
    #[must_use]
    pub fn is_valid_for(&self, leeway: Duration) -> bool {
        self.expires_at
            .duration_since(SystemTime::now())
            .is_ok_and(|remaining| remaining > leeway)
    }

    /// Returns the `Authorization` header value.
    #[must_use]
    pub fn authorization_header(&self) -> String {
        let token_type = if self.token_type.is_empty() {
            TOKEN_TYPE
        } else {
            &self.token_type
        };
        format!("{token_type} {}", self.access_token)
    }
}

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AccessToken")
            .field("access_token", &"<redacted>")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// A source of access tokens.
///
/// Implement this trait to integrate another credential store, a caching proxy,
/// or a test double.
#[async_trait]
pub trait TokenSource: Send + Sync {
    /// Returns a valid token, refreshing it when necessary.
    ///
    /// # Errors
    ///
    /// Returns an error when a token cannot be obtained.
    async fn token(&self) -> Result<AccessToken>;

    /// Marks the current token as invalid.
    ///
    /// The next call to [`TokenSource::token`] should fetch a fresh token.
    fn invalidate(&self) {}
}

/// A token source for QQ Bot application credentials.
///
/// Tokens are cached and concurrent callers share a single refresh. The source
/// never logs or exposes the application secret.
#[derive(Debug)]
pub struct QqBotTokenSource {
    credentials: Credentials,
    client: reqwest::Client,
    token_url: String,
    cached: Mutex<Option<AccessToken>>,
    invalidated: AtomicBool,
}

impl QqBotTokenSource {
    /// Creates a token source with a default HTTP client.
    #[must_use]
    pub fn new(credentials: Credentials) -> Self {
        Self {
            credentials,
            client: reqwest::Client::new(),
            token_url: DEFAULT_TOKEN_URL.to_owned(),
            cached: Mutex::new(None),
            invalidated: AtomicBool::new(false),
        }
    }

    /// Replaces the HTTP client.
    #[must_use]
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    /// Replaces the token endpoint. This is primarily useful for testing or
    /// for a compatible private gateway.
    #[must_use]
    pub fn with_token_url(mut self, token_url: impl Into<String>) -> Self {
        self.token_url = token_url.into();
        self
    }

    /// Returns the application ID.
    #[must_use]
    pub fn app_id(&self) -> &str {
        &self.credentials.app_id
    }

    /// Returns a shared reference to the source.
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    async fn refresh(&self, previous: Option<&AccessToken>) -> Result<AccessToken> {
        let response = self
            .client
            .post(&self.token_url)
            .json(&TokenRequest {
                app_id: &self.credentials.app_id,
                client_secret: &self.credentials.app_secret,
            })
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(Error::Token(format!("HTTP {status}: {body}")));
        }

        let response: TokenResponse = serde_json::from_str(&body)
            .map_err(|error| Error::Token(format!("invalid token response: {error}")))?;
        if response.code != 0 {
            return Err(Error::Token(format!(
                "{}: {}",
                response.code, response.message
            )));
        }
        if response.access_token.is_empty() {
            return Err(Error::Token(
                "token response did not contain a token".to_owned(),
            ));
        }

        if let Some(previous) = previous {
            tracing::debug!(
                old_expiry = ?previous.expires_at,
                "refreshed QQ Bot access token"
            );
        }

        let expires_in = response
            .expires_in
            .as_ref()
            .and_then(ExpiresIn::as_u64)
            .unwrap_or(0);
        Ok(AccessToken::new(
            response.access_token,
            if response.token_type.is_empty() {
                TOKEN_TYPE
            } else {
                &response.token_type
            },
            expires_in,
            SystemTime::now() + Duration::from_secs(expires_in),
        ))
    }

    /// Spawns a background refresh task.
    ///
    /// The task refreshes the token shortly before expiry and stops when the
    /// returned guard is dropped or [`TokenRefresh::abort`] is called. Calling
    /// this outside a Tokio runtime returns an inactive guard instead of
    /// panicking.
    #[must_use]
    pub fn start_refresh(self: &Arc<Self>) -> TokenRefresh {
        let source = Arc::clone(self);
        let task = async move {
            let mut consecutive_failures = 0_u32;
            loop {
                let delay = match source.token().await {
                    Ok(token) => {
                        consecutive_failures = 0;
                        token
                            .expires_at()
                            .duration_since(SystemTime::now())
                            .unwrap_or_default()
                            .saturating_sub(Duration::from_secs(9))
                    }
                    Err(error) => {
                        consecutive_failures = consecutive_failures.saturating_add(1);
                        tracing::warn!(%error, "failed to refresh QQ Bot access token");
                        Duration::from_secs(u64::from(consecutive_failures.min(30)))
                    }
                };
                tokio::time::sleep(delay.max(Duration::from_secs(1))).await;
                let mut cached = source.cached.lock().await;
                cached.take();
            }
        };
        let handle = tokio::runtime::Handle::try_current().map_or_else(
            |_| {
                tracing::warn!("token refresh was not started: no Tokio runtime");
                None
            },
            |runtime| Some(runtime.spawn(task)),
        );
        TokenRefresh { handle }
    }
}

#[async_trait]
impl TokenSource for QqBotTokenSource {
    // The cache lock is intentionally held across the refresh request so that
    // concurrent callers share a single token fetch.
    #[allow(clippy::significant_drop_tightening)]
    async fn token(&self) -> Result<AccessToken> {
        let mut cached = self.cached.lock().await;
        if !self.invalidated.load(Ordering::Acquire) {
            if let Some(token) = cached.as_ref().filter(|token| token.is_valid()) {
                return Ok(token.clone());
            }
        }
        let previous = cached.take();
        let token = self.refresh(previous.as_ref()).await?;
        *cached = Some(token.clone());
        self.invalidated.store(false, Ordering::Release);
        Ok(token)
    }

    fn invalidate(&self) {
        self.invalidated.store(true, Ordering::Release);
    }
}

/// Guard for the background token-refresh task.
///
/// Dropping the guard aborts the task.
#[derive(Debug)]
pub struct TokenRefresh {
    handle: Option<JoinHandle<()>>,
}

impl TokenRefresh {
    /// Stops the refresh task immediately.
    pub fn abort(&self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }

    /// Returns `true` when the task has finished or could not be started.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }
}

impl Drop for TokenRefresh {
    fn drop(&mut self) {
        self.abort();
    }
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    #[serde(rename = "appId")]
    app_id: &'a str,
    #[serde(rename = "clientSecret")]
    client_secret: &'a str,
}

#[derive(Deserialize)]
struct TokenResponse {
    #[serde(default)]
    code: i32,
    #[serde(default)]
    message: String,
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    token_type: String,
    #[serde(default)]
    expires_in: Option<ExpiresIn>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ExpiresIn {
    Integer(u64),
    String(String),
}

impl ExpiresIn {
    fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Integer(value) => Some(*value),
            Self::String(value) => value.parse().ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[test]
    fn credentials_debug_redacts_secret() {
        let credentials = Credentials::new("app", "super-secret");
        let debug = format!("{credentials:?}");
        assert!(!debug.contains("super-secret"));
    }

    #[test]
    fn access_token_debug_redacts_token() {
        let token = AccessToken::new(
            "secret",
            TOKEN_TYPE,
            100,
            SystemTime::now() + Duration::from_secs(100),
        );
        assert!(!format!("{token:?}").contains("secret"));
        assert_eq!(token.authorization_header(), "QQBot secret");
    }

    #[tokio::test]
    async fn fetches_and_caches_a_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "abc",
                "expires_in": "7200",
                "code": 0,
            })))
            .expect(1)
            .mount(&server)
            .await;

        let source = QqBotTokenSource::new(Credentials::new("app", "secret"))
            .with_http_client(reqwest::Client::new())
            .with_token_url(format!("{}/token", server.uri()));

        let first = source.token().await.unwrap();
        let second = source.token().await.unwrap();
        assert_eq!(first.as_str(), "abc");
        assert_eq!(second.as_str(), "abc");
        assert_eq!(first.expires_in(), 7200);
    }

    #[tokio::test]
    async fn invalidation_forces_a_refresh() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "abc",
                "expires_in": 7200,
                "code": 0,
            })))
            .expect(2)
            .mount(&server)
            .await;

        let source = QqBotTokenSource::new(Credentials::new("app", "secret"))
            .with_token_url(format!("{}/token", server.uri()));

        source.token().await.unwrap();
        source.invalidate();
        source.token().await.unwrap();
    }
}
