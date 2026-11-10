//! Opt-in live tests against the QQ sandbox.
//!
//! These tests send real active messages and are ignored by default. Run them
//! only with a sandbox bot and a target that is allowed to receive messages:
//!
//! ```text
//! $env:QQ_BOT_APP_ID = "..."
//! $env:QQ_BOT_APP_SECRET = "..."
//! $env:QQ_BOT_GROUP_OPENID = "..."
//! $env:QQ_BOT_USER_OPENID = "..."
//! cargo test --test sandbox_live -- --ignored --nocapture
//! ```

#![allow(clippy::unwrap_used, clippy::panic)]

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use botgo::models::MessageToCreate;
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{OpenApi, Result};

fn api() -> Result<OpenApi> {
    let app_id = std::env::var("QQ_BOT_APP_ID")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_ID is not set"))?;
    let app_secret = std::env::var("QQ_BOT_APP_SECRET")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_SECRET is not set"))?;
    let token_source = Arc::new(QqBotTokenSource::new(Credentials::new(&app_id, app_secret)));
    OpenApi::sandbox(app_id, token_source)
}

fn target(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
}

fn text(label: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("botgo-rs {label} sandbox test #{nonce}")
}

#[tokio::test]
#[ignore = "sends a real active message; requires sandbox credentials and QQ_BOT_GROUP_OPENID"]
async fn sandbox_group_message() {
    let message = api()
        .unwrap()
        .post_group_message(
            &target("QQ_BOT_GROUP_OPENID"),
            MessageToCreate {
                content: text("group"),
                ..MessageToCreate::default()
            },
        )
        .await
        .unwrap();

    assert_ne!(message.id, "");
    println!("group message id: {}", message.id);
}

#[tokio::test]
#[ignore = "sends a real active message; requires sandbox credentials and QQ_BOT_USER_OPENID"]
async fn sandbox_c2c_message() {
    let message = api()
        .unwrap()
        .post_c2c_message(
            &target("QQ_BOT_USER_OPENID"),
            MessageToCreate {
                content: text("c2c"),
                ..MessageToCreate::default()
            },
        )
        .await
        .unwrap();

    assert_ne!(message.id, "");
    println!("c2c message id: {}", message.id);
}
