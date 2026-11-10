//! Error types for the SDK.

use reqwest::StatusCode;

/// A convenience alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// An error returned by the QQ Bot OpenAPI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// HTTP status code.
    pub status: StatusCode,
    /// Legacy error code.
    pub code: Option<i32>,
    /// Current error code.
    pub err_code: Option<i32>,
    /// Human-readable error message.
    pub message: String,
    /// Server trace ID.
    pub trace_id: String,
    /// Raw response body.
    pub body: String,
}

impl ApiError {
    /// Creates an API error from a response.
    #[must_use]
    pub fn from_body(status: StatusCode, body: String) -> Self {
        #[derive(serde::Deserialize)]
        struct ErrorBody {
            #[serde(default)]
            message: String,
            #[serde(default)]
            code: Option<i32>,
            #[serde(default)]
            err_code: Option<i32>,
            #[serde(default)]
            trace_id: String,
        }

        let parsed = serde_json::from_str::<ErrorBody>(&body).ok();
        let (message, code, err_code, trace_id) = parsed.map_or_else(
            || (body.clone(), None, None, String::new()),
            |error| (error.message, error.code, error.err_code, error.trace_id),
        );

        Self {
            status,
            code,
            err_code,
            message,
            trace_id,
            body,
        }
    }

    /// Returns `true` when the error indicates an expired or missing token.
    #[must_use]
    pub fn is_token_expired(&self) -> bool {
        const TOKEN_EXPIRED: i32 = 11_244;
        self.code == Some(TOKEN_EXPIRED) || self.err_code == Some(TOKEN_EXPIRED)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "HTTP {}: {} (code: {:?}, err_code: {:?}, trace_id: {})",
            self.status.as_u16(),
            self.message,
            self.code,
            self.err_code,
            self.trace_id
        )
    }
}

impl std::error::Error for ApiError {}

/// The SDK error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error response from the OpenAPI.
    #[error("{0}")]
    Api(#[from] ApiError),
    /// An HTTP transport error.
    #[error("HTTP transport failed: {0}")]
    Http(#[from] reqwest::Error),
    /// A JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// A WebSocket transport error.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    /// The token source could not provide a token.
    #[error("token acquisition failed: {0}")]
    Token(String),
    /// Invalid SDK configuration.
    #[error("invalid configuration: {0}")]
    Config(String),
    /// The gateway or HTTP callback sent an unexpected payload.
    #[error("protocol error: {0}")]
    Protocol(String),
    /// A webhook signature could not be verified.
    #[error("signature verification failed: {0}")]
    Signature(String),
    /// The gateway session cannot be resumed and must be identified again.
    #[error("session cannot be resumed: {0}")]
    Session(String),
    /// The gateway closed with an unrecoverable status.
    #[error("gateway closed: {0}")]
    GatewayClosed(String),
}

impl Error {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Creates a protocol error.
    #[must_use]
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol(message.into())
    }

    /// Returns the underlying API error, if any.
    #[must_use]
    pub const fn as_api_error(&self) -> Option<&ApiError> {
        match self {
            Self::Api(error) => Some(error),
            _ => None,
        }
    }
}

/// WebSocket close codes used by the QQ Bot gateway.
pub mod close_code {
    /// Unknown backend error.
    pub const UNKNOWN_ERROR: u16 = 4000;
    /// Unknown opcode.
    pub const UNKNOWN_OPCODE: u16 = 4001;
    /// Decode error.
    pub const DECODE_ERROR: u16 = 4002;
    /// Not authenticated.
    pub const NOT_AUTHENTICATED: u16 = 4003;
    /// Authentication failed.
    pub const AUTHENTICATION_FAILED: u16 = 4004;
    /// Already authenticated.
    pub const ALREADY_AUTHENTICATED: u16 = 4005;
    /// Session no longer valid.
    pub const SESSION_NO_LONGER_VALID: u16 = 4006;
    /// Invalid sequence number.
    pub const INVALID_SEQ: u16 = 4007;
    /// Rate limited.
    pub const RATE_LIMITED: u16 = 4008;
    /// Session timed out.
    pub const SESSION_TIMEOUT: u16 = 4009;
    /// Invalid shard.
    pub const INVALID_SHARD: u16 = 4010;
    /// Sharding required.
    pub const SHARDING_REQUIRED: u16 = 4011;
    /// Invalid API version.
    pub const INVALID_API_VERSION: u16 = 4012;
    /// Invalid intents.
    pub const INVALID_INTENTS: u16 = 4013;
    /// Disallowed intents.
    pub const DISALLOWED_INTENTS: u16 = 4014;
    /// Bot is offline.
    pub const BOT_OFFLINE: u16 = 4914;
    /// Bot is banned.
    pub const BOT_BANNED: u16 = 4915;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_api_error_body() {
        let error = ApiError::from_body(
            StatusCode::BAD_REQUEST,
            r#"{"message":"bad","code":1,"err_code":2,"trace_id":"t"}"#.to_owned(),
        );
        assert_eq!(error.code, Some(1));
        assert_eq!(error.err_code, Some(2));
        assert_eq!(error.trace_id, "t");
    }

    #[test]
    fn detects_expired_token() {
        let error = ApiError::from_body(
            StatusCode::UNAUTHORIZED,
            r#"{"message":"expired","err_code":11244}"#.to_owned(),
        );
        assert!(error.is_token_expired());
    }
}
