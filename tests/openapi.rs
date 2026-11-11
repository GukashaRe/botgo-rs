#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use botgo::models::{Message, MessageToCreate, MessagesPager};
use botgo::{AccessToken, Error, OpenApi, OpenApiBuilder, Result, TokenSource};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct StaticToken;

#[async_trait]
impl TokenSource for StaticToken {
    async fn token(&self) -> Result<AccessToken> {
        Ok(AccessToken::new(
            "token",
            "QQBot",
            3600,
            SystemTime::now() + Duration::from_secs(3600),
        ))
    }
}

fn api(server: &MockServer) -> OpenApi {
    OpenApiBuilder::new("app-id", Arc::new(StaticToken))
        .base_url(server.uri())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

#[tokio::test]
async fn post_message_sends_auth_and_parses_the_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/channels/channel-1/messages"))
        .and(header("authorization", "QQBot token"))
        .and(header("x-union-appid", "app-id"))
        .and(header("user-agent", botgo::USER_AGENT))
        .and(body_json(serde_json::json!({ "content": "hello" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "message-1",
            "channel_id": "channel-1",
            "content": "hello",
            "author": { "id": "bot-1", "username": "bot" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let message: Message = api(&server)
        .post_message(
            "channel-1",
            &MessageToCreate {
                content: "hello".to_owned(),
                ..MessageToCreate::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(message.id, "message-1");
    assert_eq!(message.author.unwrap().id, "bot-1");
}

#[tokio::test]
async fn messages_encodes_pager_query_parameters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/channels/channel-1/messages"))
        .and(query_param("limit", "10"))
        .and(query_param("around", "message-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": "message-1", "channel_id": "channel-1", "content": "one" }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let messages = api(&server)
        .messages(
            "channel-1",
            &MessagesPager::new().around("message-2").limit(10),
        )
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, "message-1");
}

#[tokio::test]
async fn api_errors_include_status_code_and_trace_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/@me"))
        .respond_with(
            ResponseTemplate::new(400)
                .insert_header("X-Tps-trace-ID", "trace-1")
                .set_body_json(serde_json::json!({
                    "message": "bad request",
                    "code": 400,
                    "err_code": 40001,
                    "trace_id": "trace-1"
                })),
        )
        .mount(&server)
        .await;

    let api = api(&server);
    let error = api.me().await.unwrap_err();
    let api_error = error.as_api_error().unwrap();
    assert_eq!(api_error.status.as_u16(), 400);
    assert_eq!(api_error.code, Some(400));
    assert_eq!(api_error.err_code, Some(40_001));
    assert_eq!(api.trace_id(), "trace-1");
}

#[tokio::test]
async fn message_compatibility_unwraps_message_field() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/channels/channel-1/messages/message-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": {
                "id": "message-1",
                "channel_id": "channel-1",
                "content": "wrapped"
            }
        })))
        .mount(&server)
        .await;

    let message = api(&server)
        .message("channel-1", "message-1")
        .await
        .unwrap();
    assert_eq!(message.content, "wrapped");
}

#[tokio::test]
async fn transport_rejects_invalid_urls() {
    let server = MockServer::start().await;
    let error = api(&server)
        .transport(reqwest::Method::GET, "http://[invalid", None)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Config(_)));
}
