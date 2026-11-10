#![allow(clippy::unwrap_used)]

use botgo::models::{OpCode, WebhookValidationResponse};
use botgo::signature;
use botgo::webhook;
use serde_json::json;

fn sign_body(secret: &str, timestamp: &str, body: &[u8]) -> String {
    signature::sign(secret, timestamp, body).unwrap()
}

#[test]
fn validates_a_webhook_url() {
    let secret = "test-secret";
    let timestamp = "1728981195";
    let body = serde_json::to_vec(&json!({
        "op": 13,
        "d": {
            "plain_token": "plain-token",
            "event_ts": "1728981195"
        }
    }))
    .unwrap();
    let signature = sign_body(secret, timestamp, &body);

    let response = webhook::process(secret, &signature, timestamp, &body).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.event.is_none());

    let validation: WebhookValidationResponse = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(validation.plain_token, "plain-token");
    signature::verify(secret, &validation.signature, "1728981195", b"plain-token").unwrap();
}

#[test]
fn acknowledges_a_dispatch_and_returns_the_event() {
    let secret = "test-secret";
    let timestamp = "1728981195";
    let body = serde_json::to_vec(&json!({
        "op": 0,
        "s": 7,
        "t": "AT_MESSAGE_CREATE",
        "id": "event-9",
        "d": {
            "id": "message-1",
            "channel_id": "channel-1",
            "content": "<@!bot> hello",
            "author": { "id": "user-1", "username": "tester" }
        }
    }))
    .unwrap();
    let signature = sign_body(secret, timestamp, &body);

    let response = webhook::process(secret, &signature, timestamp, &body).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body_str(), r#"{"op":12,"d":0}"#);
    assert_eq!(response.event_id.as_deref(), Some("event-9"));
    assert_eq!(response.seq, Some(7));
    let event = response.event.unwrap();
    assert_eq!(event.event_type().as_str(), "AT_MESSAGE_CREATE");
}

#[test]
fn rejects_a_tampered_body() {
    let secret = "test-secret";
    let timestamp = "1728981195";
    let body = br#"{"op":1,"d":1}"#;
    let signature = sign_body(secret, timestamp, body);

    let error = webhook::process(secret, &signature, timestamp, br#"{"op":1,"d":2}"#).unwrap_err();
    assert!(matches!(error, botgo::Error::Signature(_)));
}

#[test]
fn acknowledges_heartbeats() {
    let secret = "test-secret";
    let timestamp = "1728981195";
    let body = br#"{"op":1,"d":42}"#;
    let signature = sign_body(secret, timestamp, body);

    let response = webhook::process(secret, &signature, timestamp, body).unwrap();
    assert_eq!(response.body_str(), r#"{"op":11,"d":42}"#);
    assert_eq!(
        serde_json::from_slice::<botgo::models::WebhookAck>(&response.body)
            .unwrap()
            .op,
        OpCode::HEARTBEAT_ACK.0
    );
}
