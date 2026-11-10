#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use botgo::models::{
    EventType, GroupBlacklistUpdate, GroupRobotEvent, Intents, StreamMessageRequest, WsPayload,
};
use botgo::{AccessToken, Event, OpenApiBuilder, Result, TokenSource};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
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
async fn group_info_and_members_use_v2_paths() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/groups/group-1/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "group_openid": "group-1",
            "group_name": "读书会",
            "group_member_num": 42,
            "group_tags": ["阅读"]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/groups/group-1/members"))
        .and(query_param("cursor", "next"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "members": [{
                "member_openid": "member-1",
                "username": "小明",
                "member_role": "admin",
                "bot": false
            }],
            "next_cursor": ""
        })))
        .expect(1)
        .mount(&server)
        .await;

    let api = api(&server);
    let info = api.group_info("group-1").await.unwrap();
    assert_eq!(info.group_name, "读书会");
    assert_eq!(info.group_member_num, 42);

    let members = api.group_members("group-1", Some("next")).await.unwrap();
    assert_eq!(members.members[0].member_role, "admin");
}

#[tokio::test]
async fn group_blacklist_operations_use_the_documented_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/groups/group-1/member_blacklist"))
        .and(body_json(json!({
            "op": "add",
            "member_openids": ["member-1"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "fail_openids": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = api(&server)
        .update_group_blacklist(
            "group-1",
            &GroupBlacklistUpdate {
                op: "add".to_owned(),
                member_openids: vec!["member-1".to_owned()],
            },
        )
        .await
        .unwrap();
    assert_eq!(result.fail_openids, Vec::<String>::new());
}

#[tokio::test]
async fn rich_media_prepare_and_stream_message_are_supported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/groups/group-1/upload_prepare"))
        .and(body_json(json!({
            "file_type": 2,
            "file_size": "1024",
            "file_name": "demo.mp4",
            "md5": "md5",
            "sha1": "sha1",
            "md5_10m": "md5-10m"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "upload_id": "upload-1",
            "block_size": "5242880",
            "parts": [{
                "index": 0,
                "presigned_url": "https://example.com/part",
                "block_size": "1024"
            }],
            "upload_config": {
                "concurrency": 1,
                "retry_timeout": 300,
                "retry_delay": 1
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v2/users/user-1/stream_messages"))
        .and(body_json(json!({
            "input_state": 1,
            "content_type": "markdown",
            "content_raw": "生成中",
            "msg_id": "msg-1",
            "msg_seq": 1
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "stream-1",
            "timestamp": "2026-07-21T10:00:00+08:00",
            "remain_msg_len": 100
        })))
        .expect(1)
        .mount(&server)
        .await;

    let api = api(&server);
    let prepared = api
        .prepare_group_upload(
            "group-1",
            &botgo::models::UploadPrepareRequest {
                file_type: 2,
                file_size: "1024".to_owned(),
                file_name: "demo.mp4".to_owned(),
                md5: "md5".to_owned(),
                sha1: "sha1".to_owned(),
                md5_10m: "md5-10m".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(prepared.upload_id, "upload-1");
    assert_eq!(prepared.parts.len(), 1);

    let stream_request = StreamMessageRequest {
        input_state: 1,
        content_type: "markdown".to_owned(),
        content_raw: "生成中".to_owned(),
        msg_id: "msg-1".to_owned(),
        msg_seq: 1,
        ..StreamMessageRequest::default()
    };
    eprintln!(
        "stream body: {}",
        serde_json::to_string(&stream_request).unwrap()
    );
    let result = api.stream_c2c_message("user-1", &stream_request).await;
    if let Err(error) = &result {
        for request in server.received_requests().await.unwrap() {
            eprintln!("request: {} {}", request.method, request.url);
        }
        eprintln!("stream error: {error}");
    }
    let streamed = result.unwrap();
    assert_eq!(streamed.id, "stream-1");
    assert_eq!(streamed.remain_msg_len, 100);
}

#[test]
fn new_group_events_parse_into_typed_payloads() {
    let payload: WsPayload = serde_json::from_value(json!({
        "op": 0,
        "t": "GROUP_ADD_ROBOT",
        "d": {
            "timestamp": 1_784_570_534,
            "group_openid": "group-1",
            "op_member_openid": "member-1"
        }
    }))
    .unwrap();
    match Event::from_payload(&payload).unwrap() {
        Event::GroupAddRobot(event) => {
            assert_eq!(event.group_openid, "group-1");
            assert_eq!(event.op_member_openid, "member-1");
        }
        other => panic!("expected GROUP_ADD_ROBOT, got {other:?}"),
    }

    let payload: WsPayload = serde_json::from_value(json!({
        "op": 0,
        "t": "GROUP_JOIN_REQUEST",
        "d": {
            "group_openid": "group-1",
            "join_request_id": "request-1",
            "member_openid": "member-1",
            "username": "小明",
            "apply_at": "2026-08-05T16:21:40+08:00",
            "apply_source": "self_apply",
            "verify_info": {
                "method": "admin_review_qa",
                "review_qa_list": [{"question": "q", "answer": "a"}]
            },
            "auto_approved": {"strategy_id": "strategy-1"}
        }
    }))
    .unwrap();
    match Event::from_payload(&payload).unwrap() {
        Event::GroupJoinRequest(event) => {
            assert_eq!(event.request.join_request_id, "request-1");
            assert_eq!(
                event.request.verify_info.unwrap().review_qa_list[0].answer,
                "a"
            );
            assert_eq!(
                event.request.auto_approved.unwrap().strategy_id,
                "strategy-1"
            );
        }
        other => panic!("expected GROUP_JOIN_REQUEST, got {other:?}"),
    }
}

#[test]
fn group_message_create_parses_v2_message_fields() {
    let payload: WsPayload = serde_json::from_value(json!({
        "op": 0,
        "t": "GROUP_MESSAGE_CREATE",
        "d": {
            "id": "message-1",
            "group_openid": "group-1",
            "content": "hello",
            "message_type": 3,
            "timestamp": "2026-07-21T10:00:00+08:00",
            "author": {
                "id": "member-1",
                "member_openid": "member-1",
                "member_role": "member",
                "username": "小明"
            },
            "attachments": [{
                "url": "https://example.com/a.jpg",
                "content_type": "image/jpeg",
                "voice_wav_url": "https://example.com/a.wav",
                "asr_refer_text": "hello"
            }],
            "ark_data": {
                "ark_type": "miniapp",
                "prompt": "prompt",
                "fields": {"title": "card"}
            },
            "message_scene": {
                "source": "default",
                "ext": ["msg_idx=REFIDX_1"]
            },
            "msg_elements": [{
                "msg_idx": "REFIDX_1",
                "message_type": 103,
                "content": "quoted"
            }]
        }
    }))
    .unwrap();

    match Event::from_payload(&payload).unwrap() {
        Event::GroupMessageCreate(message) => {
            assert_eq!(message.group_openid, "group-1");
            assert_eq!(message.message_type, 3);
            assert_eq!(message.author.unwrap().member_role, "member");
            assert_eq!(message.attachments[0].asr_refer_text, "hello");
            assert_eq!(message.ark_data.unwrap().ark_type, "miniapp");
            assert_eq!(message.message_scene.unwrap().ext[0], "msg_idx=REFIDX_1");
            assert_eq!(message.msg_elements[0].content, "quoted");
        }
        other => panic!("expected GROUP_MESSAGE_CREATE, got {other:?}"),
    }
}

#[tokio::test]
async fn menu_and_panel_endpoints_use_documented_payloads() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/menu"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "version": 1,
            "menu": {"items": [{"name": "帮助", "type": "send_message", "send_message": "/help"}]}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v2/menu"))
        .and(body_json(json!({
            "menu": {"items": [{"name": "帮助", "type": "send_message", "send_message": "/help"}]}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"version": 2})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v2/panels"))
        .and(body_json(json!({
            "scope": "c2c",
            "target_type": "all",
            "panel": {
                "items": [{"name": "天气", "desc": "查询天气", "type": "command"}]
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"panel_id": "panel-1"})))
        .expect(1)
        .mount(&server)
        .await;

    let api = api(&server);
    let current = api.menu().await.unwrap();
    assert_eq!(current.version, 1);
    assert_eq!(current.menu.unwrap().items[0].name, "帮助");

    let updated = api.update_menu(&current_menu()).await.unwrap();
    assert_eq!(updated.version, 2);

    let created = api
        .create_panel(&botgo::models::PanelCreate {
            scope: "c2c".to_owned(),
            target_type: "all".to_owned(),
            panel: botgo::models::Panel {
                items: vec![botgo::models::PanelItem {
                    name: "天气".to_owned(),
                    desc: "查询天气".to_owned(),
                    item_type: "command".to_owned(),
                    ..botgo::models::PanelItem::default()
                }],
                ..botgo::models::Panel::default()
            },
            ..botgo::models::PanelCreate::default()
        })
        .await
        .unwrap();
    assert_eq!(created.panel_id, "panel-1");
}

fn current_menu() -> botgo::models::Menu {
    botgo::models::Menu {
        items: vec![botgo::models::MenuItem {
            name: "帮助".to_owned(),
            item_type: "send_message".to_owned(),
            send_message: "/help".to_owned(),
            ..botgo::models::MenuItem::default()
        }],
    }
}

#[tokio::test]
async fn join_approval_strategy_and_group_mute_endpoints() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/groups/join_approval_strategy"))
        .and(body_json(json!({
            "group_openids": ["group-1"],
            "is_enable": "on"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "strategy_id": "strategy-1",
            "is_enable": "on",
            "expire_at": "2027-08-05T15:30:16+08:00"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/groups/group-1/restrict_chat_setting"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "global_rule": {
                "mode": "none",
                "schedule_rules": [],
                "recurring_rules": []
            },
            "members": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let api = api(&server);
    let strategy = api
        .create_join_approval_strategy(&botgo::models::CreateJoinApprovalStrategy {
            group_openids: vec!["group-1".to_owned()],
            is_enable: "on".to_owned(),
            ..botgo::models::CreateJoinApprovalStrategy::default()
        })
        .await
        .unwrap();
    assert_eq!(strategy.strategy_id, "strategy-1");

    let setting = api.group_restrict_chat_setting("group-1").await.unwrap();
    assert_eq!(setting.global_rule.mode, "none");
    assert_eq!(
        setting.members,
        Vec::<botgo::models::MemberMuteState>::new()
    );
}

#[test]
fn group_member_event_intent_is_separate_from_message_intent() {
    assert_eq!(
        EventType::GROUP_MEMBER_ADD.intent(),
        Some(Intents::GROUP_MEMBER_EVENT)
    );
    assert_eq!(
        EventType::GROUP_MESSAGE_CREATE.intent(),
        Some(Intents::GROUP_AND_C2C_EVENT)
    );
    assert!(Intents::ALL.contains(Intents::GROUP_MEMBER_EVENT));
    let _ = GroupRobotEvent::default();
}
