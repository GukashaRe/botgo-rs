//! Framework-agnostic HTTP webhook handling.

use serde_json::Value;

use crate::error::{Error, Result};
use crate::event::Event;
use crate::models::{
    OpCode, WebhookAck, WebhookValidationRequest, WebhookValidationResponse, WsPayload,
};
use crate::signature;

/// Result of processing a webhook request.
#[derive(Debug)]
pub struct WebhookResponse {
    /// Suggested HTTP status code.
    pub status: u16,
    /// Response body.
    pub body: Vec<u8>,
    /// Parsed event for dispatch callbacks.
    pub event: Option<Event>,
    /// Outer event ID for `event_id` passive replies.
    pub event_id: Option<String>,
    /// Gateway sequence number for dispatch callbacks.
    pub seq: Option<u64>,
    /// Non-fatal event parsing error.
    pub error: Option<String>,
}

impl WebhookResponse {
    /// Returns the body as UTF-8 when possible.
    #[must_use]
    pub fn body_str(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap_or_default()
    }
}

/// Verifies and processes a raw webhook callback.
///
/// `signature` and `timestamp` are the values of the
/// [`signature::SIGNATURE_HEADER`] and [`signature::TIMESTAMP_HEADER`] headers.
/// The returned status is `401` for an invalid signature and `200` for all
/// signed payloads, including malformed dispatches that must be negatively
/// acknowledged.
///
/// # Errors
///
/// Returns an error only for an invalid signature or malformed validation
/// payload. Dispatch parsing errors are reported through
/// [`WebhookResponse::error`] and a negative acknowledgement.
pub fn process(
    secret: &str,
    signature_header: &str,
    timestamp: &str,
    body: &[u8],
) -> Result<WebhookResponse> {
    signature::verify(secret, signature_header, timestamp, body)?;

    let payload: WsPayload = serde_json::from_slice(body)?;
    match payload.op {
        OpCode::HTTP_CALLBACK_VALIDATION => {
            let request: WebhookValidationRequest = payload
                .deserialize_data()
                .map_err(|error| Error::Protocol(format!("invalid validation payload: {error}")))?;
            let ack = gen_validation_ack(secret, &request)?;
            Ok(WebhookResponse {
                status: 200,
                body: ack,
                event: None,
                event_id: None,
                seq: None,
                error: None,
            })
        }
        OpCode::HEARTBEAT => {
            let sequence = payload.data.as_ref().and_then(Value::as_u64).unwrap_or(0);
            Ok(WebhookResponse {
                status: 200,
                body: gen_heartbeat_ack(sequence).into_bytes(),
                event: None,
                event_id: None,
                seq: None,
                error: None,
            })
        }
        OpCode::DISPATCH => match Event::from_payload(&payload) {
            Ok(event) => Ok(WebhookResponse {
                status: 200,
                body: gen_dispatch_ack(true).into_bytes(),
                event: Some(event),
                event_id: payload.event_id,
                seq: payload.seq,
                error: None,
            }),
            Err(error) => {
                tracing::warn!(%error, "failed to parse webhook dispatch event");
                Ok(WebhookResponse {
                    status: 200,
                    body: gen_dispatch_ack(false).into_bytes(),
                    event: None,
                    event_id: payload.event_id,
                    seq: payload.seq,
                    error: Some(error.to_string()),
                })
            }
        },
        other => Ok(WebhookResponse {
            status: 200,
            body: Vec::new(),
            event: None,
            event_id: None,
            seq: None,
            error: Some(format!("unsupported webhook opcode {other}")),
        }),
    }
}

/// Builds the heartbeat acknowledgement body.
#[must_use]
pub fn gen_heartbeat_ack(sequence: u64) -> String {
    serde_json::to_string(&WebhookAck {
        op: OpCode::HEARTBEAT_ACK.0,
        d: u32::try_from(sequence).unwrap_or(u32::MAX),
    })
    .unwrap_or_else(|_| r#"{"op":11,"d":0}"#.to_owned())
}

/// Builds the dispatch acknowledgement body.
///
/// A `d` value of `1` asks the platform to retry the event.
#[must_use]
pub fn gen_dispatch_ack(success: bool) -> String {
    serde_json::to_string(&WebhookAck {
        op: OpCode::HTTP_CALLBACK_ACK.0,
        d: u32::from(!success),
    })
    .unwrap_or_else(|_| r#"{"op":12,"d":1}"#.to_owned())
}

/// Builds a URL-validation response.
///
/// # Errors
///
/// Returns an error when the secret is empty.
pub fn gen_validation_ack(secret: &str, request: &WebhookValidationRequest) -> Result<Vec<u8>> {
    let signature = signature::sign(secret, &request.event_ts, request.plain_token.as_bytes())?;
    serde_json::to_vec(&WebhookValidationResponse {
        plain_token: request.plain_token.clone(),
        signature,
        data_version: String::new(),
    })
    .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledgements_match_the_protocol() {
        assert_eq!(gen_heartbeat_ack(1314), r#"{"op":11,"d":1314}"#);
        assert_eq!(gen_dispatch_ack(true), r#"{"op":12,"d":0}"#);
        assert_eq!(gen_dispatch_ack(false), r#"{"op":12,"d":1}"#);
    }

    #[test]
    fn rejects_an_invalid_signature() {
        let response = process("secret", "00", "1", b"{}");
        assert!(response.is_err());
    }
}
