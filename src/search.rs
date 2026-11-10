//! Simulate an inline-search interaction against a local callback server.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::models::{
    Interaction, InteractionData, SearchInputResolved, SearchResponse, interaction_data_type,
    interaction_type,
};
use crate::signature;

/// Maximum response size accepted by the simulator.
pub const MAX_RESPONSE_BYTES: usize = 65_535;

/// Configuration for [`simulate_search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchConfig {
    /// Application ID.
    pub app_id: String,
    /// Callback endpoint.
    pub endpoint: String,
    /// Application secret used to sign the request.
    pub secret: String,
}

impl SearchConfig {
    /// Creates a configuration.
    #[must_use]
    pub fn new(
        app_id: impl Into<String>,
        endpoint: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            endpoint: endpoint.into(),
            secret: secret.into(),
        }
    }
}

/// Sends a simulated inline-search interaction.
///
/// # Errors
///
/// Returns an error when the request, signature, or response is invalid.
pub async fn simulate_search(config: &SearchConfig, keyword: &str) -> Result<SearchResponse> {
    let resolved = serde_json::to_value(SearchInputResolved {
        keyword: keyword.to_owned(),
    })?;
    let interaction = Interaction {
        application_id: config.app_id.clone(),
        interaction_type: interaction_type::COMMAND,
        data: Some(InteractionData {
            name: "search".to_owned(),
            data_type: interaction_data_type::CHAT_SEARCH,
            resolved: Some(resolved),
        }),
        version: 1,
        ..Interaction::default()
    };
    let body = serde_json::to_vec(&interaction)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::Protocol(format!("system clock error: {error}")))?
        .as_secs()
        .to_string();
    let signature = signature::sign(&config.secret, &timestamp, &body)?;

    let response = reqwest::Client::new()
        .post(&config.endpoint)
        .header(signature::TIMESTAMP_HEADER, &timestamp)
        .header(signature::SIGNATURE_HEADER, signature)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await?;

    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        return Err(Error::Protocol(format!(
            "search callback returned HTTP {status}"
        )));
    }
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(Error::Protocol(format!(
            "search callback response exceeds {MAX_RESPONSE_BYTES} bytes"
        )));
    }
    serde_json::from_slice(&bytes).map_err(Error::from)
}
