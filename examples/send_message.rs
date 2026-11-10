//! Send a message to a channel using credentials from environment variables.
//!
//! Required variables: `QQ_BOT_APP_ID`, `QQ_BOT_APP_SECRET`, `QQ_BOT_CHANNEL_ID`.

use std::sync::Arc;

use botgo::models::MessageToCreate;
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{OpenApi, Result};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let app_id = std::env::var("QQ_BOT_APP_ID")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_ID is not set"))?;
    let app_secret = std::env::var("QQ_BOT_APP_SECRET")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_SECRET is not set"))?;
    let channel_id = std::env::var("QQ_BOT_CHANNEL_ID")
        .map_err(|_| botgo::Error::config("QQ_BOT_CHANNEL_ID is not set"))?;

    let token_source = Arc::new(QqBotTokenSource::new(Credentials::new(&app_id, app_secret)));
    let _refresh = token_source.start_refresh();
    let api = OpenApi::new(app_id, token_source)?;

    let message = api
        .post_message(
            &channel_id,
            &MessageToCreate {
                content: "hello from botgo-rs".to_owned(),
                ..MessageToCreate::default()
            },
        )
        .await?;

    println!("sent message {}", message.id);
    Ok(())
}
