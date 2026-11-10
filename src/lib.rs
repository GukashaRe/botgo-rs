//! An idiomatic Rust SDK for the Tencent QQ Bot OpenAPI and gateway.
//!
//! `botgo` is a Rust port of the official Go SDK
//! [`tencent-connect/botgo`](https://github.com/tencent-connect/botgo). It
//! provides:
//!
//! - an async OpenAPI client covering guilds, channels, messages, members,
//!   roles, permissions, announcements, schedules, pins, reactions, and the
//!   HTTP gateway;
//! - a WebSocket gateway client with sharding, heartbeats, and resume support;
//! - Ed25519 webhook signature verification and callback acknowledgements;
//! - strongly typed models for every public payload in the Go SDK.
//!
//! # Quick start
//!
//! ```no_run
//! use std::sync::Arc;
//!
//! use botgo::models::MessageToCreate;
//! use botgo::token::{Credentials, QqBotTokenSource};
//! use botgo::OpenApi;
//!
//! # async fn run() -> Result<(), botgo::Error> {
//! let credentials = Credentials::new("app-id", "app-secret");
//! let token_source = Arc::new(QqBotTokenSource::new(credentials));
//! let api = OpenApi::new("app-id", token_source)?;
//!
//! let message = api
//!     .post_message(
//!         "channel-id",
//!         &MessageToCreate {
//!             content: "hello from Rust".to_owned(),
//!             ..MessageToCreate::default()
//!         },
//!     )
//!     .await?;
//! println!("sent {}", message.id);
//! # Ok(())
//! # }
//! ```
//!
//! # Design notes
//!
//! Unlike the Go SDK, this crate does not use package-level mutable state:
//! clients, token sources, and gateway connections are explicit values. Events
//! are delivered as a typed stream instead of through global callback
//! registries, which makes concurrent bots and tests straightforward.

#![forbid(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

pub mod constants;
pub mod error;
pub mod event;
pub mod gateway;
pub mod message;
pub mod models;
pub mod openapi;
pub mod search;
pub mod signature;
pub mod token;
pub mod webhook;

pub use error::{ApiError, Error, Result};
pub use event::Event;
pub use gateway::{EventStream, Gateway, GatewayEvent};
pub use openapi::{OpenApi, OpenApiBuilder, RequestOptions};
pub use token::{AccessToken, Credentials, QqBotTokenSource, TokenSource};

/// The SDK version reported in the `User-Agent` header.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
