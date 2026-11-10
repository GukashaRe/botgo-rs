//! Listen for gateway events and print messages that mention the bot.
//!
//! Required variables: `QQ_BOT_APP_ID`, `QQ_BOT_APP_SECRET`.

use std::sync::Arc;

use botgo::gateway::{Gateway, GatewayEvent};
use botgo::models::Intents;
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{Event, OpenApi, Result};
use futures_util::StreamExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let app_id = std::env::var("QQ_BOT_APP_ID")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_ID is not set"))?;
    let app_secret = std::env::var("QQ_BOT_APP_SECRET")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_SECRET is not set"))?;

    let token_source = Arc::new(QqBotTokenSource::new(Credentials::new(&app_id, app_secret)));
    let _refresh = token_source.start_refresh();
    let api = OpenApi::new(
        &app_id,
        Arc::clone(&token_source) as Arc<dyn botgo::TokenSource>,
    )?;

    let intents = Intents::GUILDS | Intents::GUILD_AT_MESSAGE | Intents::GROUP_MESSAGES;
    let mut events = Gateway::new(api, token_source, intents).events()?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            item = events.next() => {
                match item {
                    Some(Ok(GatewayEvent::Ready { data, shard })) => {
                        println!("shard {}/{} ready as {}", shard.shard_id + 1, shard.shard_count, data.user.username);
                    }
                    Some(Ok(GatewayEvent::Dispatch { event: Event::AtMessageCreate(message), .. })) => {
                        println!("channel {}: {}", message.channel_id, message.content);
                    }
                    Some(Ok(GatewayEvent::Dispatch { event: Event::C2cMessageCreate(message), .. })) => {
                        println!("c2c: {}", message.content);
                    }
                    Some(Ok(GatewayEvent::Dispatch { event: Event::GroupAtMessageCreate(message), .. })) => {
                        println!("group {}: {}", message.group_id, message.content);
                    }
                    Some(Ok(GatewayEvent::Reconnecting { reason, delay_seconds })) => {
                        eprintln!("reconnecting in {delay_seconds}s: {reason}");
                    }
                    Some(Ok(GatewayEvent::Dispatch { event, .. })) => {
                        println!("event {}", event.event_type());
                    }
                    Some(Ok(GatewayEvent::HeartbeatAck)) => {}
                    Some(Err(error)) => eprintln!("gateway error: {error}"),
                    None => break,
                }
            }
        }
    }

    Ok(())
}
