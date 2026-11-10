#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use botgo::models::{EventType, Intents, MessageToCreate, RichMediaMessage, WsPayload};
use botgo::{AccessToken, Event, OpenApiBuilder, Result, TokenSource};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
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

fn api(server: &MockServer) -> botgo::OpenApi {
    OpenApiBuilder::new("app-id", Arc::new(StaticToken))
        .base_url(server.uri())
        .build()
        .unwrap()
}

#[tokio::test]
async fn group_text_message_uses_v2_group_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/groups/group-1/messages"))
        .and(body_json(json!({
            "content": "group reply",
            "msg_id": "event-1"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "out-1",
            "group_id": "group-1",
            "content": "group reply"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let message = api(&server)
        .post_group_message(
            "group-1",
            MessageToCreate {
                content: "group reply".to_owned(),
                msg_id: "event-1".to_owned(),
                ..MessageToCreate::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(message.id, "out-1");
    assert_eq!(message.group_id, "group-1");
}

#[tokio::test]
async fn c2c_rich_media_uses_v2_user_files_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/users/user-1/files"))
        .and(body_json(json!({
            "file_type": 1,
            "url": "https://example.com/image.png",
            "srv_send_msg": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "out-2",
            "content": "",
            "author": {"id": "user-1", "username": "tester"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let message = api(&server)
        .post_c2c_message(
            "user-1",
            RichMediaMessage {
                file_type: 1,
                url: "https://example.com/image.png".to_owned(),
                srv_send_msg: true,
                ..RichMediaMessage::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(message.id, "out-2");
}

#[tokio::test]
async fn group_and_c2c_messages_can_be_retracted() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v2/groups/group-1/messages/message-1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v2/users/user-1/messages/message-2"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let api = api(&server);
    api.retract_group_message("group-1", "message-1")
        .await
        .unwrap();
    api.retract_c2c_message("user-1", "message-2")
        .await
        .unwrap();
}

#[test]
fn group_and_c2c_events_are_typed() {
    let group_payload: WsPayload = serde_json::from_value(json!({
        "op": 0,
        "s": 1,
        "t": "GROUP_AT_MESSAGE_CREATE",
        "d": {
            "id": "message-1",
            "group_id": "group-1",
            "content": "<@!bot> hello",
            "author": {"id": "user-1", "username": "tester"}
        }
    }))
    .unwrap();
    let group_event = Event::from_payload(&group_payload).unwrap();
    assert_eq!(group_event.event_type(), EventType::GROUP_AT_MESSAGE_CREATE);
    match group_event {
        Event::GroupAtMessageCreate(message) => {
            assert_eq!(message.group_id, "group-1");
            assert_eq!(message.author.unwrap().id, "user-1");
        }
        other => panic!("expected group event, got {other:?}"),
    }

    let c2c_payload: WsPayload = serde_json::from_value(json!({
        "op": 0,
        "s": 2,
        "t": "C2C_MESSAGE_CREATE",
        "d": {
            "id": "message-2",
            "content": "private hello",
            "author": {"id": "user-2", "username": "tester"}
        }
    }))
    .unwrap();
    let c2c_event = Event::from_payload(&c2c_payload).unwrap();
    assert_eq!(c2c_event.event_type(), EventType::C2C_MESSAGE_CREATE);
    match c2c_event {
        Event::C2cMessageCreate(message) => {
            assert_eq!(message.content, "private hello");
            assert_eq!(message.author.unwrap().id, "user-2");
        }
        other => panic!("expected c2c event, got {other:?}"),
    }
}

#[test]
fn group_messages_intent_covers_group_and_c2c_events() {
    assert!(Intents::GROUP_MESSAGES.contains(EventType::GROUP_AT_MESSAGE_CREATE.intent().unwrap()));
    assert!(Intents::GROUP_MESSAGES.contains(EventType::C2C_MESSAGE_CREATE.intent().unwrap()));
}
