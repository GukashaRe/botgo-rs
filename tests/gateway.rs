#![allow(clippy::unwrap_used, clippy::too_many_lines)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use botgo::gateway::{Gateway, GatewayEvent};
use botgo::models::Intents;
use botgo::{AccessToken, OpenApiBuilder, Result, TokenSource};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use wiremock::matchers::{method, path};
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

#[tokio::test]
async fn gateway_identifies_and_emits_typed_events() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();

        socket
            .send(Message::Text(
                json!({"op": 10, "d": {"heartbeat_interval": 1000}})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();

        let identify = socket.next().await.unwrap().unwrap();
        let identify = identify.to_text().unwrap();
        assert!(
            identify.contains("\"op\":2"),
            "unexpected frame: {identify}"
        );
        assert!(
            identify.contains("\"intents\":1"),
            "unexpected frame: {identify}"
        );

        socket
            .send(Message::Text(
                json!({
                    "op": 0,
                    "s": 1,
                    "t": "READY",
                    "d": {
                        "version": 1,
                        "session_id": "session-1",
                        "user": {"id": "bot-1", "username": "bot", "bot": true},
                        "shard": [0, 1]
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();

        socket
            .send(Message::Ping(vec![1, 2, 3].into()))
            .await
            .unwrap();

        let pong = tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Message::Pong(payload) = socket.next().await.unwrap().unwrap() {
                    break payload;
                }
            }
        })
        .await
        .expect("gateway did not answer websocket ping");
        assert_eq!(pong.as_ref(), &[1, 2, 3]);

        socket
            .send(Message::Text(
                json!({
                    "op": 0,
                    "s": 2,
                    "t": "AT_MESSAGE_CREATE",
                    "id": "event-9",
                    "d": {
                        "id": "message-1",
                        "channel_id": "channel-1",
                        "content": "hello",
                        "author": {"id": "user-1", "username": "tester"}
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(30)).await;
    });

    let api_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/gateway/bot"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "url": format!("ws://{address}"),
            "shards": 1,
            "session_start_limit": {
                "total": 10,
                "remaining": 10,
                "reset_after": 0,
                "max_concurrency": 1
            }
        })))
        .mount(&api_server)
        .await;

    let api = OpenApiBuilder::new("app-id", Arc::new(StaticToken))
        .base_url(api_server.uri())
        .build()
        .unwrap();
    let mut events = Gateway::new(api, Arc::new(StaticToken), Intents::GUILDS)
        .with_queue_size(16)
        .events()
        .unwrap();

    let ready = tokio::time::timeout(Duration::from_secs(15), events.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match ready {
        GatewayEvent::Ready { data, shard } => {
            assert_eq!(data.session_id, "session-1");
            assert_eq!(shard.shard_id, 0);
        }
        other => panic!("expected READY, got {other:?}"),
    }

    let dispatch = tokio::time::timeout(Duration::from_secs(10), events.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match dispatch {
        GatewayEvent::Dispatch {
            event,
            event_id,
            seq,
        } => {
            assert_eq!(event.event_type().as_str(), "AT_MESSAGE_CREATE");
            assert_eq!(event_id.as_deref(), Some("event-9"));
            assert_eq!(seq, Some(2));
        }
        other => panic!("expected dispatch, got {other:?}"),
    }

    server.abort();
}
