//! WebSocket gateway client with sharding, heartbeats, and resume support.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::json;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::error::{Error, Result, close_code};
use crate::event::Event;
use crate::models::{
    Intents, OpCode, ShardConfig, WebsocketAp, WsHelloData, WsIdentityData, WsPayload, WsReadyData,
    WsResumeData,
};
use crate::openapi::OpenApi;
use crate::token::TokenSource;

type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type SocketSink = futures_util::stream::SplitSink<Socket, Message>;

/// An event emitted by the gateway.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayEvent {
    /// A shard completed the identify handshake.
    Ready {
        /// READY payload.
        data: WsReadyData,
        /// Shard that became ready.
        shard: ShardConfig,
    },
    /// A dispatch event. Unknown event types are preserved as
    /// [`Event::Unknown`].
    Dispatch {
        /// Parsed event.
        event: Event,
        /// Outer event ID used for `event_id` passive replies.
        event_id: Option<String>,
        /// Gateway sequence number.
        seq: Option<u64>,
    },
    /// The server acknowledged a heartbeat.
    HeartbeatAck,
    /// A shard is about to reconnect.
    Reconnecting {
        /// Reason for the reconnect.
        reason: String,
        /// Backoff before the reconnect, in seconds.
        delay_seconds: u64,
    },
}

/// A stream of gateway events.
///
/// Dropping the stream stops all shard tasks.
pub struct EventStream {
    receiver: mpsc::Receiver<Result<GatewayEvent>>,
}

impl Stream for EventStream {
    type Item = Result<GatewayEvent>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

/// A sharded gateway client.
pub struct Gateway {
    api: OpenApi,
    token_source: Arc<dyn TokenSource>,
    intents: Intents,
    shard: Option<ShardConfig>,
    queue_size: usize,
}

impl std::fmt::Debug for Gateway {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Gateway")
            .field("api", &self.api)
            .field("intents", &self.intents)
            .field("shard", &self.shard)
            .finish_non_exhaustive()
    }
}

impl Gateway {
    /// Creates a gateway for all shards returned by `/gateway/bot`.
    #[must_use]
    pub fn new(api: OpenApi, token_source: Arc<dyn TokenSource>, intents: Intents) -> Self {
        Self {
            api,
            token_source,
            intents,
            shard: None,
            queue_size: 1024,
        }
    }

    /// Creates a gateway using the OpenAPI client's token source.
    #[must_use]
    pub fn from_api(api: OpenApi, intents: Intents) -> Self {
        let token_source = api.token_source();
        Self::new(api, token_source, intents)
    }

    /// Limits the gateway to a single shard.
    #[must_use]
    pub const fn with_shard(mut self, shard: ShardConfig) -> Self {
        self.shard = Some(shard);
        self
    }

    /// Sets the event channel capacity.
    #[must_use]
    pub const fn with_queue_size(mut self, queue_size: usize) -> Self {
        self.queue_size = queue_size;
        self
    }

    /// Connects to the gateway and returns a stream of events.
    ///
    /// The gateway metadata is fetched in the background, so the first item may
    /// be an error if the OpenAPI request fails.
    ///
    /// # Errors
    ///
    /// Returns an error when called outside a Tokio runtime.
    pub fn events(self) -> Result<EventStream> {
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| Error::config("Gateway::events requires a Tokio runtime"))?;
        let (sender, receiver) = mpsc::channel(self.queue_size.max(1));
        handle.spawn(async move {
            if let Err(error) = self.run(sender.clone()).await {
                let _ = sender.send(Err(error)).await;
            }
        });
        Ok(EventStream { receiver })
    }

    async fn run(self, sender: mpsc::Sender<Result<GatewayEvent>>) -> Result<()> {
        let gateway = self.api.ws().await?;
        validate_session_limit(&gateway)?;
        let shard_count = gateway.shards.max(1);
        let interval = connection_interval(gateway.session_start_limit.max_concurrency);

        let shards: Vec<ShardConfig> = self.shard.map_or_else(
            || {
                (0..shard_count)
                    .map(|shard_id| ShardConfig::new(shard_id, shard_count))
                    .collect()
            },
            |shard| vec![shard],
        );

        for shard in shards {
            let sender = sender.clone();
            let token_source = Arc::clone(&self.token_source);
            let url = gateway.url.clone();
            let intents = self.intents;
            tokio::time::sleep(interval).await;
            tokio::spawn(async move {
                run_shard(url, token_source, intents, shard, sender).await;
            });
        }
        Ok(())
    }
}

/// Returns the delay between shard starts.
#[must_use]
pub fn connection_interval(max_concurrency: u32) -> Duration {
    let max_concurrency = max_concurrency.max(1);
    let seconds = (2.0_f64 / f64::from(max_concurrency)).round().max(1.0);
    Duration::from_secs_f64(seconds)
}

fn validate_session_limit(gateway: &WebsocketAp) -> Result<()> {
    if gateway.shards > gateway.session_start_limit.remaining {
        return Err(Error::Session(format!(
            "requested {} shards but only {} session starts remain",
            gateway.shards, gateway.session_start_limit.remaining
        )));
    }
    Ok(())
}

#[derive(Default)]
struct SessionState {
    session_id: Option<String>,
    last_seq: u64,
}

enum ConnectionEnd {
    Resume(String),
    Identify(String),
    Fatal(Error),
}

async fn run_shard(
    url: String,
    token_source: Arc<dyn TokenSource>,
    intents: Intents,
    shard: ShardConfig,
    sender: mpsc::Sender<Result<GatewayEvent>>,
) {
    let mut state = SessionState::default();
    let mut backoff_seconds = 1_u64;

    loop {
        let started = std::time::Instant::now();
        let end = tokio::select! {
            () = sender.closed() => return,
            end = run_connection(
                &url,
                Arc::clone(&token_source),
                intents,
                shard,
                &mut state,
                &sender,
            ) => end,
        };

        let (reset_session, reason) = match end {
            Ok(ConnectionEnd::Resume(reason)) => (false, reason),
            Ok(ConnectionEnd::Identify(reason)) => (true, reason),
            Ok(ConnectionEnd::Fatal(error)) => {
                let _ = sender.send(Err(error)).await;
                return;
            }
            Err(error) => {
                tracing::warn!(%error, shard_id = shard.shard_id, "gateway connection failed");
                (false, error.to_string())
            }
        };

        if reset_session {
            state.session_id = None;
            state.last_seq = 0;
        }
        if started.elapsed() >= Duration::from_secs(60) {
            backoff_seconds = 1;
        }

        let _ = sender
            .send(Ok(GatewayEvent::Reconnecting {
                reason,
                delay_seconds: backoff_seconds,
            }))
            .await;
        tokio::select! {
            () = sender.closed() => return,
            () = tokio::time::sleep(Duration::from_secs(backoff_seconds)) => {}
        }
        backoff_seconds = (backoff_seconds * 2).min(30);
    }
}

#[allow(clippy::too_many_lines)]
async fn run_connection(
    url: &str,
    token_source: Arc<dyn TokenSource>,
    intents: Intents,
    shard: ShardConfig,
    state: &mut SessionState,
    sender: &mpsc::Sender<Result<GatewayEvent>>,
) -> Result<ConnectionEnd> {
    let (socket, _) = tokio_tungstenite::connect_async(url).await?;
    let (sink, mut stream) = socket.split();
    let sink = Arc::new(Mutex::new(sink));
    let last_seq = Arc::new(AtomicU64::new(state.last_seq));

    let first = tokio::time::timeout(Duration::from_secs(30), stream.next())
        .await
        .map_err(|_| Error::Protocol("timed out waiting for HELLO".to_owned()))?
        .ok_or_else(|| Error::Protocol("gateway closed before HELLO".to_owned()))??;
    let hello_payload = match parse_message(first)? {
        Incoming::Payload(payload) => payload,
        Incoming::Closed(code) => {
            return Ok(ConnectionEnd::Identify(format!(
                "gateway closed before HELLO with code {code}"
            )));
        }
        Incoming::Ignored => {
            return Err(Error::Protocol(
                "gateway sent an empty frame before HELLO".to_owned(),
            ));
        }
        Incoming::Ping(_) => {
            return Err(Error::Protocol(
                "gateway sent a ping before HELLO".to_owned(),
            ));
        }
    };
    if hello_payload.op != OpCode::HELLO {
        return Err(Error::Protocol(format!(
            "expected HELLO, received {}",
            hello_payload.op
        )));
    }
    let hello: WsHelloData = hello_payload.deserialize_data()?;

    let token = token_source.token().await?;
    if let Some(session_id) = state.session_id.clone() {
        send_payload(
            &sink,
            &WsPayload::new(
                OpCode::RESUME,
                Some(json!(WsResumeData {
                    token: token.authorization_header(),
                    session_id,
                    seq: state.last_seq,
                })),
            ),
        )
        .await?;
    } else {
        send_payload(
            &sink,
            &WsPayload::new(
                OpCode::IDENTIFY,
                Some(json!(WsIdentityData {
                    token: token.authorization_header(),
                    intents,
                    shard: [shard.shard_id, shard.shard_count],
                    properties: None,
                })),
            ),
        )
        .await?;
    }

    let heartbeat = HeartbeatTask::new(start_heartbeat(
        Arc::clone(&sink),
        Duration::from_millis(hello.heartbeat_interval.max(1)),
        Arc::clone(&last_seq),
    ));

    while let Some(message) = stream.next().await {
        let message = match message {
            Ok(message) => message,
            Err(error) => {
                heartbeat.abort();
                return classify_transport_error(error);
            }
        };
        let payload = match parse_message(message) {
            Ok(Incoming::Payload(payload)) => payload,
            Ok(Incoming::Closed(code)) => {
                heartbeat.abort();
                if code == close_code::AUTHENTICATION_FAILED {
                    token_source.invalidate();
                }
                return Ok(classify_close(code));
            }
            Ok(Incoming::Ping(payload)) => {
                // tungstenite queues an automatic pong, but with a split
                // stream it may not be flushed until the next heartbeat. Reply
                // immediately so the server does not reset an otherwise idle
                // connection.
                sink.lock()
                    .await
                    .send(Message::Pong(payload.into()))
                    .await
                    .map_err(Error::from)?;
                continue;
            }
            Ok(Incoming::Ignored) => continue,
            Err(error) => {
                tracing::warn!(%error, "ignoring malformed gateway frame");
                let _ = sender.send(Err(error)).await;
                continue;
            }
        };

        if let Some(sequence) = payload.seq {
            state.last_seq = sequence;
            last_seq.store(sequence, Ordering::Relaxed);
        }

        match payload.op {
            OpCode::DISPATCH => {
                if payload.event_type.as_ref() == Some(&crate::models::EventType::READY) {
                    let ready: WsReadyData = payload.deserialize_data()?;
                    state.session_id = Some(ready.session_id.clone());
                    state.last_seq = payload.seq.unwrap_or(0);
                    last_seq.store(state.last_seq, Ordering::Relaxed);
                    let _ = sender
                        .send(Ok(GatewayEvent::Ready { data: ready, shard }))
                        .await;
                    continue;
                }
                match Event::from_payload(&payload) {
                    Ok(event) => {
                        let _ = sender
                            .send(Ok(GatewayEvent::Dispatch {
                                event,
                                event_id: payload.event_id.clone(),
                                seq: payload.seq,
                            }))
                            .await;
                    }
                    Err(error) => {
                        tracing::warn!(%error, "failed to parse gateway event");
                        let _ = sender.send(Err(error)).await;
                    }
                }
            }
            OpCode::HEARTBEAT_ACK => {
                let _ = sender.send(Ok(GatewayEvent::HeartbeatAck)).await;
            }
            OpCode::RECONNECT => {
                heartbeat.abort();
                return Ok(ConnectionEnd::Resume(
                    "server requested reconnect".to_owned(),
                ));
            }
            OpCode::INVALID_SESSION => {
                heartbeat.abort();
                return Ok(ConnectionEnd::Identify(
                    "server invalidated the session".to_owned(),
                ));
            }
            _ => {}
        }
    }

    heartbeat.abort();
    if state.session_id.is_some() {
        Ok(ConnectionEnd::Resume("gateway stream ended".to_owned()))
    } else {
        Ok(ConnectionEnd::Identify(
            "gateway stream ended before READY".to_owned(),
        ))
    }
}

async fn send_payload(sink: &Arc<Mutex<SocketSink>>, payload: &WsPayload) -> Result<()> {
    let text = serde_json::to_string(payload)?;
    sink.lock()
        .await
        .send(Message::Text(text.into()))
        .await
        .map_err(Error::from)
}

fn start_heartbeat(
    sink: Arc<Mutex<SocketSink>>,
    interval: Duration,
    last_seq: Arc<AtomicU64>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // The first interval tick fires immediately; consume it so the first
        // heartbeat is sent after one full heartbeat interval, as expected by
        // the gateway.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let payload = WsPayload::new(
                OpCode::HEARTBEAT,
                Some(json!(last_seq.load(Ordering::Relaxed))),
            );
            if let Err(error) = send_payload(&sink, &payload).await {
                tracing::warn!(%error, "failed to send gateway heartbeat");
                return;
            }
        }
    })
}

struct HeartbeatTask {
    handle: JoinHandle<()>,
}

impl HeartbeatTask {
    const fn new(handle: JoinHandle<()>) -> Self {
        Self { handle }
    }

    fn abort(&self) {
        self.handle.abort();
    }
}

impl Drop for HeartbeatTask {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

enum Incoming {
    Payload(WsPayload),
    Closed(u16),
    Ping(Vec<u8>),
    Ignored,
}

fn parse_message(message: Message) -> Result<Incoming> {
    let bytes = match message {
        Message::Text(text) => text.as_bytes().to_vec(),
        Message::Binary(bytes) => bytes.to_vec(),
        Message::Close(frame) => {
            let code = frame.as_ref().map_or(0, |frame| u16::from(frame.code));
            return Ok(Incoming::Closed(code));
        }
        Message::Ping(payload) => return Ok(Incoming::Ping(payload.to_vec())),
        Message::Pong(_) | Message::Frame(_) => {
            return Ok(Incoming::Ignored);
        }
    };
    serde_json::from_slice(&bytes)
        .map(Incoming::Payload)
        .map_err(Error::from)
}

fn classify_transport_error(error: tokio_tungstenite::tungstenite::Error) -> Result<ConnectionEnd> {
    if matches!(
        &error,
        tokio_tungstenite::tungstenite::Error::Protocol(
            tokio_tungstenite::tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
        )
    ) {
        return Ok(ConnectionEnd::Resume(
            "connection reset without closing handshake".to_owned(),
        ));
    }

    if let tokio_tungstenite::tungstenite::Error::Io(io) = &error {
        let retryable_kind = matches!(
            io.kind(),
            std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::UnexpectedEof
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::TimedOut
        );
        let retryable_os = matches!(io.raw_os_error(), Some(10053 | 10054 | 10060));
        if retryable_kind || retryable_os {
            return Ok(ConnectionEnd::Resume(format!(
                "gateway connection interrupted: {io}"
            )));
        }
    }

    Err(Error::WebSocket(error))
}

fn classify_close(code: u16) -> ConnectionEnd {
    match code {
        close_code::BOT_OFFLINE => {
            ConnectionEnd::Fatal(Error::GatewayClosed("bot is offline".to_owned()))
        }
        close_code::BOT_BANNED => {
            ConnectionEnd::Fatal(Error::GatewayClosed("bot is banned".to_owned()))
        }
        close_code::AUTHENTICATION_FAILED => {
            ConnectionEnd::Identify("authentication failed".to_owned())
        }
        close_code::SESSION_NO_LONGER_VALID
        | close_code::INVALID_SEQ
        | close_code::INVALID_SHARD
        | close_code::SHARDING_REQUIRED => {
            ConnectionEnd::Identify(format!("session closed with code {code}"))
        }
        close_code::SESSION_TIMEOUT => {
            ConnectionEnd::Resume("session timed out; resuming".to_owned())
        }
        _ => ConnectionEnd::Resume(format!("gateway closed with code {code}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_matches_the_go_sdk() {
        assert_eq!(connection_interval(0), Duration::from_secs(2));
        assert_eq!(connection_interval(1), Duration::from_secs(2));
        assert_eq!(connection_interval(2), Duration::from_secs(1));
    }

    #[test]
    fn close_codes_are_classified() {
        assert!(matches!(
            classify_close(close_code::BOT_BANNED),
            ConnectionEnd::Fatal(_)
        ));
        assert!(matches!(
            classify_close(close_code::SESSION_NO_LONGER_VALID),
            ConnectionEnd::Identify(_)
        ));
        assert!(matches!(
            classify_close(close_code::SESSION_TIMEOUT),
            ConnectionEnd::Resume(_)
        ));
    }

    #[test]
    fn windows_connection_reset_is_resumable() {
        let error =
            tokio_tungstenite::tungstenite::Error::Io(std::io::Error::from_raw_os_error(10054));
        assert!(matches!(
            classify_transport_error(error),
            Ok(ConnectionEnd::Resume(_))
        ));
    }
}
