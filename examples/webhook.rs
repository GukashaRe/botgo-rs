//! Verify and process a webhook request in any HTTP framework.
//!
//! The SDK is framework-agnostic: read the raw body and the two signature
//! headers, call [`botgo::webhook::process`], and return the suggested status
//! and body from your handler.

use botgo::{signature, webhook};

fn main() -> botgo::Result<()> {
    let secret = "demo-secret";
    let timestamp = "1728981195";
    let body = br#"{"op":1,"d":42}"#;
    let signature = signature::sign(secret, timestamp, body)?;

    match webhook::process(secret, &signature, timestamp, body) {
        Ok(response) => {
            println!("status: {}", response.status);
            println!("body: {}", response.body_str());
            if let Some(event) = response.event {
                println!("event: {}", event.event_type());
            }
        }
        Err(error) => eprintln!("invalid callback: {error}"),
    }
    Ok(())
}
