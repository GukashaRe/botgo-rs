//! Sandbox smoke test for QQ group and C2C active messages.
//!
//! Required environment variables:
//! - `QQ_BOT_APP_ID`
//! - `QQ_BOT_APP_SECRET`
//!
//! Optional targets:
//! - `QQ_BOT_GROUP_OPENID`
//! - `QQ_BOT_USER_OPENID`
//!
//! Run with:
//! `cargo run --example sandbox_check`
//!
//! At least one target must be set. The messages sent here are active messages,
//! so they consume active-message quota and require the corresponding sandbox
//! relationship/permissions. For passive replies, use `echo.rs` instead.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use botgo::models::MessageToCreate;
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{Error, OpenApi, Result};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let app_id = required("QQ_BOT_APP_ID")?;
    let app_secret = required("QQ_BOT_APP_SECRET")?;
    let group_openid = optional("QQ_BOT_GROUP_OPENID");
    let user_openid = optional("QQ_BOT_USER_OPENID");

    if group_openid.is_none() && user_openid.is_none() {
        return Err(Error::config(
            "set QQ_BOT_GROUP_OPENID and/or QQ_BOT_USER_OPENID",
        ));
    }

    let token_source = Arc::new(QqBotTokenSource::new(Credentials::new(&app_id, app_secret)));
    let api = OpenApi::sandbox(&app_id, token_source)?;

    let me = api.me().await?;
    println!("sandbox connected as {} ({})", me.username, me.id);

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::protocol(format!("system clock error: {error}")))?
        .as_secs();
    let content = format!("botgo-rs sandbox check #{nonce}");

    if let Some(group_openid) = group_openid {
        let message = api
            .post_group_message(
                &group_openid,
                MessageToCreate {
                    content: content.clone(),
                    ..MessageToCreate::default()
                },
            )
            .await?;
        println!("group message sent: id={}", message.id);
    }

    if let Some(user_openid) = user_openid {
        let message = api
            .post_c2c_message(
                &user_openid,
                MessageToCreate {
                    content,
                    ..MessageToCreate::default()
                },
            )
            .await?;
        println!("c2c message sent: id={}", message.id);
    }

    println!("sandbox check passed");
    Ok(())
}

fn required(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| Error::config(format!("{name} is not set")))
}

fn optional(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
