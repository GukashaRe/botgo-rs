//! Minimal sandbox echo bot for QQ groups and C2C chats.
//!
//! Required environment variables:
//! - `QQ_BOT_APP_ID`
//! - `QQ_BOT_APP_SECRET`
//!
//! Run with:
//! `cargo run --example echo`
//!
//! For SDK debug logs:
//! `RUST_LOG=botgo=debug cargo run --example echo`
//!
//! This example always uses the QQ sandbox OpenAPI domain. It replies to
//! `GROUP_AT_MESSAGE_CREATE` and `C2C_MESSAGE_CREATE` with the same text.

use std::sync::Arc;

use botgo::gateway::{Gateway, GatewayEvent};
use botgo::models::{Intents, Message, MessageToCreate};
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{Event, OpenApi, Result};
use futures_util::StreamExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_tracing();

    let app_id = std::env::var("QQ_BOT_APP_ID")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_ID is not set"))?;
    let app_secret = std::env::var("QQ_BOT_APP_SECRET")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_SECRET is not set"))?;

    let token_source = Arc::new(QqBotTokenSource::new(Credentials::new(&app_id, app_secret)));
    let _refresh = token_source.start_refresh();
    let api = OpenApi::sandbox(&app_id, token_source)?;

    // GROUP_MESSAGES covers GROUP_AT_MESSAGE_CREATE and C2C_MESSAGE_CREATE.
    let mut events = Gateway::from_api(api.clone(), Intents::GROUP_MESSAGES).events()?;

    println!("sandbox echo bot is running");

    while let Some(item) = events.next().await {
        match item? {
            GatewayEvent::Ready { data, .. } => {
                println!("gateway ready as {}", data.user.username);
            }
            GatewayEvent::Dispatch { event, .. } => {
                if let Err(error) = echo(&api, event).await {
                    eprintln!("echo failed: {error}");
                }
            }
            GatewayEvent::Reconnecting { reason, .. } => {
                eprintln!("gateway reconnecting: {reason}");
            }
            GatewayEvent::HeartbeatAck => {}
        }
    }

    Ok(())
}

async fn echo(api: &OpenApi, event: Event) -> Result<()> {
    match event {
        Event::GroupAtMessageCreate(message) => {
            let group_id = group_id(&message);
            println!("group {}: {}", group_id, message.content);
            api.post_group_message(group_id, reply(&message)).await?;
        }
        Event::C2cMessageCreate(message) => {
            let Some(author) = &message.author else {
                return Ok(());
            };
            let user_id = if author.user_openid.is_empty() {
                &author.id
            } else {
                &author.user_openid
            };
            println!("c2c {}: {}", user_id, message.content);
            api.post_c2c_message(user_id, reply(&message)).await?;
        }
        _ => {}
    }
    Ok(())
}

fn group_id(message: &Message) -> &str {
    if message.group_openid.is_empty() {
        &message.group_id
    } else {
        &message.group_openid
    }
}

fn reply(message: &Message) -> MessageToCreate {
    MessageToCreate {
        content: message.content.clone(),
        msg_id: message.id.clone(),
        ..MessageToCreate::default()
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("botgo=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}
