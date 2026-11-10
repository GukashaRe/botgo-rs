//! Protocol constants shared by the OpenAPI and gateway clients.

/// Production OpenAPI domain.
pub const API_DOMAIN: &str = "https://api.sgroup.qq.com";
/// Sandbox OpenAPI domain.
pub const SANDBOX_API_DOMAIN: &str = "https://sandbox.api.sgroup.qq.com";
/// Token service domain.
pub const TOKEN_DOMAIN: &str = "https://bots.qq.com";
/// Default token endpoint.
pub const TOKEN_URL: &str = "https://bots.qq.com/app/getAppAccessToken";
/// Header carrying the server trace ID.
pub const TRACE_ID_HEADER: &str = "X-Tps-trace-ID";
/// Header carrying the application ID.
pub const UNION_APP_ID_HEADER: &str = "X-Union-Appid";
/// Token type used by the QQ Bot OpenAPI.
pub const TOKEN_TYPE: &str = "QQBot";
/// Maximum number of idle connections used by the Go SDK.
pub const MAX_IDLE_CONNECTIONS: usize = 3000;
