//! Gateway protocol payloads, intents, opcodes, and event names.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Event types delivered by the QQ Bot gateway.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventType(Cow<'static, str>);

impl EventType {
    /// `READY`
    pub const READY: Self = Self(Cow::Borrowed("READY"));
    /// `GUILD_CREATE`
    pub const GUILD_CREATE: Self = Self(Cow::Borrowed("GUILD_CREATE"));
    /// `GUILD_UPDATE`
    pub const GUILD_UPDATE: Self = Self(Cow::Borrowed("GUILD_UPDATE"));
    /// `GUILD_DELETE`
    pub const GUILD_DELETE: Self = Self(Cow::Borrowed("GUILD_DELETE"));
    /// `CHANNEL_CREATE`
    pub const CHANNEL_CREATE: Self = Self(Cow::Borrowed("CHANNEL_CREATE"));
    /// `CHANNEL_UPDATE`
    pub const CHANNEL_UPDATE: Self = Self(Cow::Borrowed("CHANNEL_UPDATE"));
    /// `CHANNEL_DELETE`
    pub const CHANNEL_DELETE: Self = Self(Cow::Borrowed("CHANNEL_DELETE"));
    /// `GUILD_MEMBER_ADD`
    pub const GUILD_MEMBER_ADD: Self = Self(Cow::Borrowed("GUILD_MEMBER_ADD"));
    /// `GUILD_MEMBER_UPDATE`
    pub const GUILD_MEMBER_UPDATE: Self = Self(Cow::Borrowed("GUILD_MEMBER_UPDATE"));
    /// `GUILD_MEMBER_REMOVE`
    pub const GUILD_MEMBER_REMOVE: Self = Self(Cow::Borrowed("GUILD_MEMBER_REMOVE"));
    /// `MESSAGE_CREATE`
    pub const MESSAGE_CREATE: Self = Self(Cow::Borrowed("MESSAGE_CREATE"));
    /// `MESSAGE_DELETE`
    pub const MESSAGE_DELETE: Self = Self(Cow::Borrowed("MESSAGE_DELETE"));
    /// `PUBLIC_MESSAGE_DELETE`
    pub const PUBLIC_MESSAGE_DELETE: Self = Self(Cow::Borrowed("PUBLIC_MESSAGE_DELETE"));
    /// `MESSAGE_REACTION_ADD`
    pub const MESSAGE_REACTION_ADD: Self = Self(Cow::Borrowed("MESSAGE_REACTION_ADD"));
    /// `MESSAGE_REACTION_REMOVE`
    pub const MESSAGE_REACTION_REMOVE: Self = Self(Cow::Borrowed("MESSAGE_REACTION_REMOVE"));
    /// `AT_MESSAGE_CREATE`
    pub const AT_MESSAGE_CREATE: Self = Self(Cow::Borrowed("AT_MESSAGE_CREATE"));
    /// `DIRECT_MESSAGE_CREATE`
    pub const DIRECT_MESSAGE_CREATE: Self = Self(Cow::Borrowed("DIRECT_MESSAGE_CREATE"));
    /// `DIRECT_MESSAGE_DELETE`
    pub const DIRECT_MESSAGE_DELETE: Self = Self(Cow::Borrowed("DIRECT_MESSAGE_DELETE"));
    /// `AUDIO_START`
    pub const AUDIO_START: Self = Self(Cow::Borrowed("AUDIO_START"));
    /// `AUDIO_FINISH`
    pub const AUDIO_FINISH: Self = Self(Cow::Borrowed("AUDIO_FINISH"));
    /// `AUDIO_ON_MIC`
    pub const AUDIO_ON_MIC: Self = Self(Cow::Borrowed("AUDIO_ON_MIC"));
    /// `AUDIO_OFF_MIC`
    pub const AUDIO_OFF_MIC: Self = Self(Cow::Borrowed("AUDIO_OFF_MIC"));
    /// `MESSAGE_AUDIT_PASS`
    pub const MESSAGE_AUDIT_PASS: Self = Self(Cow::Borrowed("MESSAGE_AUDIT_PASS"));
    /// `MESSAGE_AUDIT_REJECT`
    pub const MESSAGE_AUDIT_REJECT: Self = Self(Cow::Borrowed("MESSAGE_AUDIT_REJECT"));
    /// `FORUM_THREAD_CREATE`
    pub const FORUM_THREAD_CREATE: Self = Self(Cow::Borrowed("FORUM_THREAD_CREATE"));
    /// `FORUM_THREAD_UPDATE`
    pub const FORUM_THREAD_UPDATE: Self = Self(Cow::Borrowed("FORUM_THREAD_UPDATE"));
    /// `FORUM_THREAD_DELETE`
    pub const FORUM_THREAD_DELETE: Self = Self(Cow::Borrowed("FORUM_THREAD_DELETE"));
    /// `FORUM_POST_CREATE`
    pub const FORUM_POST_CREATE: Self = Self(Cow::Borrowed("FORUM_POST_CREATE"));
    /// `FORUM_POST_DELETE`
    pub const FORUM_POST_DELETE: Self = Self(Cow::Borrowed("FORUM_POST_DELETE"));
    /// `FORUM_REPLY_CREATE`
    pub const FORUM_REPLY_CREATE: Self = Self(Cow::Borrowed("FORUM_REPLY_CREATE"));
    /// `FORUM_REPLY_DELETE`
    pub const FORUM_REPLY_DELETE: Self = Self(Cow::Borrowed("FORUM_REPLY_DELETE"));
    /// `FORUM_PUBLISH_AUDIT_RESULT`
    pub const FORUM_PUBLISH_AUDIT_RESULT: Self = Self(Cow::Borrowed("FORUM_PUBLISH_AUDIT_RESULT"));
    /// `INTERACTION_CREATE`
    pub const INTERACTION_CREATE: Self = Self(Cow::Borrowed("INTERACTION_CREATE"));
    /// `GROUP_AT_MESSAGE_CREATE`
    pub const GROUP_AT_MESSAGE_CREATE: Self = Self(Cow::Borrowed("GROUP_AT_MESSAGE_CREATE"));
    /// `GROUP_MESSAGE_CREATE`
    pub const GROUP_MESSAGE_CREATE: Self = Self(Cow::Borrowed("GROUP_MESSAGE_CREATE"));
    /// `C2C_MESSAGE_CREATE`
    pub const C2C_MESSAGE_CREATE: Self = Self(Cow::Borrowed("C2C_MESSAGE_CREATE"));
    /// `GROUP_ADD_ROBOT`
    pub const GROUP_ADD_ROBOT: Self = Self(Cow::Borrowed("GROUP_ADD_ROBOT"));
    /// `GROUP_DEL_ROBOT`
    pub const GROUP_DEL_ROBOT: Self = Self(Cow::Borrowed("GROUP_DEL_ROBOT"));
    /// `GROUP_JOIN_REQUEST`
    pub const GROUP_JOIN_REQUEST: Self = Self(Cow::Borrowed("GROUP_JOIN_REQUEST"));
    /// `GROUP_MEMBER_ADD`
    pub const GROUP_MEMBER_ADD: Self = Self(Cow::Borrowed("GROUP_MEMBER_ADD"));
    /// `GROUP_MEMBER_REMOVE`
    pub const GROUP_MEMBER_REMOVE: Self = Self(Cow::Borrowed("GROUP_MEMBER_REMOVE"));
    /// `GROUP_MSG_RECEIVE`
    pub const GROUP_MSG_RECEIVE: Self = Self(Cow::Borrowed("GROUP_MSG_RECEIVE"));
    /// `GROUP_MSG_REJECT`
    pub const GROUP_MSG_REJECT: Self = Self(Cow::Borrowed("GROUP_MSG_REJECT"));
    /// `C2C_MSG_RECEIVE`
    pub const C2C_MSG_RECEIVE: Self = Self(Cow::Borrowed("C2C_MSG_RECEIVE"));
    /// `C2C_MSG_REJECT`
    pub const C2C_MSG_REJECT: Self = Self(Cow::Borrowed("C2C_MSG_REJECT"));
    /// `SUBSCRIBE_MESSAGE_STATUS`
    pub const SUBSCRIBE_MESSAGE_STATUS: Self = Self(Cow::Borrowed("SUBSCRIBE_MESSAGE_STATUS"));
    /// `FRIEND_ADD`
    pub const FRIEND_ADD: Self = Self(Cow::Borrowed("FRIEND_ADD"));
    /// `FRIEND_DEL`
    pub const FRIEND_DEL: Self = Self(Cow::Borrowed("FRIEND_DEL"));
    /// `ENTER_AIO`
    pub const ENTER_AIO: Self = Self(Cow::Borrowed("ENTER_AIO"));

    /// Creates an event type from a string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    /// Returns the wire representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the intent that delivers this event.
    #[must_use]
    pub fn intent(&self) -> Option<Intents> {
        Some(match self.as_str() {
            "GUILD_CREATE" | "GUILD_UPDATE" | "GUILD_DELETE" | "CHANNEL_CREATE"
            | "CHANNEL_UPDATE" | "CHANNEL_DELETE" => Intents::GUILDS,
            "GUILD_MEMBER_ADD" | "GUILD_MEMBER_UPDATE" | "GUILD_MEMBER_REMOVE" => {
                Intents::GUILD_MEMBERS
            }
            "MESSAGE_CREATE" | "MESSAGE_DELETE" => Intents::GUILD_MESSAGES,
            "AT_MESSAGE_CREATE" | "PUBLIC_MESSAGE_DELETE" => Intents::GUILD_AT_MESSAGE,
            "MESSAGE_REACTION_ADD" | "MESSAGE_REACTION_REMOVE" => Intents::GUILD_MESSAGE_REACTIONS,
            "DIRECT_MESSAGE_CREATE" | "DIRECT_MESSAGE_DELETE" => Intents::DIRECT_MESSAGES,
            "AUDIO_START" | "AUDIO_FINISH" | "AUDIO_ON_MIC" | "AUDIO_OFF_MIC" => Intents::AUDIO,
            "MESSAGE_AUDIT_PASS" | "MESSAGE_AUDIT_REJECT" => Intents::AUDIT,
            "FORUM_THREAD_CREATE"
            | "FORUM_THREAD_UPDATE"
            | "FORUM_THREAD_DELETE"
            | "FORUM_POST_CREATE"
            | "FORUM_POST_DELETE"
            | "FORUM_REPLY_CREATE"
            | "FORUM_REPLY_DELETE"
            | "FORUM_PUBLISH_AUDIT_RESULT" => Intents::FORUM,
            "INTERACTION_CREATE" => Intents::INTERACTION,
            "GROUP_AT_MESSAGE_CREATE"
            | "GROUP_MESSAGE_CREATE"
            | "C2C_MESSAGE_CREATE"
            | "GROUP_ADD_ROBOT"
            | "GROUP_DEL_ROBOT"
            | "GROUP_MSG_RECEIVE"
            | "GROUP_MSG_REJECT"
            | "C2C_MSG_RECEIVE"
            | "C2C_MSG_REJECT"
            | "SUBSCRIBE_MESSAGE_STATUS"
            | "FRIEND_ADD"
            | "FRIEND_DEL" => Intents::GROUP_MESSAGES,
            "GROUP_JOIN_REQUEST" | "GROUP_MEMBER_ADD" | "GROUP_MEMBER_REMOVE" => {
                Intents::GROUP_MEMBER_EVENT
            }
            "ENTER_AIO" => Intents::ENTER_AIO,
            _ => return None,
        })
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for EventType {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for EventType {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from)
    }
}

/// Gateway intents, a compact bit set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Intents(u32);

impl Intents {
    /// No events.
    pub const NONE: Self = Self(0);
    /// Guild and channel lifecycle events.
    pub const GUILDS: Self = Self(1 << 0);
    /// Guild member events.
    pub const GUILD_MEMBERS: Self = Self(1 << 1);
    /// Guild ban events.
    pub const GUILD_BANS: Self = Self(1 << 2);
    /// Guild emoji events.
    pub const GUILD_EMOJIS: Self = Self(1 << 3);
    /// Guild integration events.
    pub const GUILD_INTEGRATIONS: Self = Self(1 << 4);
    /// Guild webhook events.
    pub const GUILD_WEBHOOKS: Self = Self(1 << 5);
    /// Guild invite events.
    pub const GUILD_INVITES: Self = Self(1 << 6);
    /// Guild voice-state events.
    pub const GUILD_VOICE_STATES: Self = Self(1 << 7);
    /// Guild presence events.
    pub const GUILD_PRESENCES: Self = Self(1 << 8);
    /// Guild message events.
    pub const GUILD_MESSAGES: Self = Self(1 << 9);
    /// Guild reaction events.
    pub const GUILD_MESSAGE_REACTIONS: Self = Self(1 << 10);
    /// Guild typing events.
    pub const GUILD_MESSAGE_TYPING: Self = Self(1 << 11);
    /// Direct-message events.
    pub const DIRECT_MESSAGES: Self = Self(1 << 12);
    /// Direct-message reaction events.
    pub const DIRECT_MESSAGE_REACTIONS: Self = Self(1 << 13);
    /// Direct-message typing events.
    pub const DIRECT_MESSAGE_TYPING: Self = Self(1 << 14);
    /// AIO entry events.
    pub const ENTER_AIO: Self = Self(1 << 23);
    /// Group member events.
    pub const GROUP_MEMBER_EVENT: Self = Self(1 << 24);
    /// Group and C2C message events.
    pub const GROUP_MESSAGES: Self = Self(1 << 25);
    /// Alias for the official `GROUP_AND_C2C_EVENT` intent name.
    pub const GROUP_AND_C2C_EVENT: Self = Self::GROUP_MESSAGES;
    /// Interaction events.
    pub const INTERACTION: Self = Self(1 << 26);
    /// Audit events.
    pub const AUDIT: Self = Self(1 << 27);
    /// Forum events.
    pub const FORUM: Self = Self(1 << 28);
    /// Audio events.
    pub const AUDIO: Self = Self(1 << 29);
    /// Guild `@` message events.
    pub const GUILD_AT_MESSAGE: Self = Self(1 << 30);

    /// Every intent supported by the SDK.
    pub const ALL: Self = Self(
        Self::GUILDS.0
            | Self::GUILD_MEMBERS.0
            | Self::GUILD_BANS.0
            | Self::GUILD_EMOJIS.0
            | Self::GUILD_INTEGRATIONS.0
            | Self::GUILD_WEBHOOKS.0
            | Self::GUILD_INVITES.0
            | Self::GUILD_VOICE_STATES.0
            | Self::GUILD_PRESENCES.0
            | Self::GUILD_MESSAGES.0
            | Self::GUILD_MESSAGE_REACTIONS.0
            | Self::GUILD_MESSAGE_TYPING.0
            | Self::DIRECT_MESSAGES.0
            | Self::DIRECT_MESSAGE_REACTIONS.0
            | Self::DIRECT_MESSAGE_TYPING.0
            | Self::ENTER_AIO.0
            | Self::GROUP_MEMBER_EVENT.0
            | Self::GROUP_MESSAGES.0
            | Self::INTERACTION.0
            | Self::AUDIT.0
            | Self::FORUM.0
            | Self::AUDIO.0
            | Self::GUILD_AT_MESSAGE.0,
    );

    /// Creates an intent set from raw bits.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the raw bits.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns `true` if every bit in `other` is present.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if no bits are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Unions two intent sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Intersects two intent sets.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Adds another intent set in place.
    pub const fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes another intent set in place.
    pub const fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl std::ops::BitOr for Intents {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Intents {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Intents {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// A gateway opcode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpCode(pub i32);

impl OpCode {
    /// Dispatch event.
    pub const DISPATCH: Self = Self(0);
    /// Heartbeat.
    pub const HEARTBEAT: Self = Self(1);
    /// Identify.
    pub const IDENTIFY: Self = Self(2);
    /// Resume.
    pub const RESUME: Self = Self(6);
    /// Reconnect.
    pub const RECONNECT: Self = Self(7);
    /// Invalid session.
    pub const INVALID_SESSION: Self = Self(9);
    /// Hello.
    pub const HELLO: Self = Self(10);
    /// Heartbeat acknowledgement.
    pub const HEARTBEAT_ACK: Self = Self(11);
    /// HTTP callback acknowledgement.
    pub const HTTP_CALLBACK_ACK: Self = Self(12);
    /// HTTP callback URL validation.
    pub const HTTP_CALLBACK_VALIDATION: Self = Self(13);

    /// Returns a human-readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self.0 {
            0 => "Event",
            1 => "Heartbeat",
            2 => "Identity",
            6 => "Resume",
            7 => "Reconnect",
            9 => "InvalidSession",
            10 => "Hello",
            11 => "HeartbeatAck",
            12 => "HttpCallbackAck",
            13 => "HttpCallbackValidation",
            _ => "Unknown",
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// A raw gateway frame.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsPayload {
    /// Opcode.
    #[serde(rename = "op")]
    pub op: OpCode,
    /// Sequence number.
    #[serde(rename = "s", default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    /// Event type for dispatch frames.
    #[serde(rename = "t", default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    /// Event ID.
    #[serde(rename = "id", default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Event data.
    #[serde(rename = "d", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl WsPayload {
    /// Creates an outgoing frame.
    #[must_use]
    pub fn new(op: OpCode, data: Option<serde_json::Value>) -> Self {
        Self {
            op,
            data,
            ..Self::default()
        }
    }

    /// Deserializes `d` into a concrete type.
    ///
    /// # Errors
    ///
    /// Returns an error when the frame has no data or the data does not match
    /// the requested type.
    pub fn deserialize_data<T>(&self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.data.clone().unwrap_or(serde_json::Value::Null))
    }
}

/// Identity payload sent after `HELLO`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsIdentityData {
    /// Authorization token, in `"QQBot <access_token>"` form.
    pub token: String,
    /// Subscribed intents.
    pub intents: Intents,
    /// `[shard_id, shard_count]`.
    pub shard: [u32; 2],
    /// Client properties.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<WsProperties>,
}

/// Client properties sent during identify.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsProperties {
    /// Operating system.
    #[serde(rename = "$os", skip_serializing_if = "String::is_empty")]
    pub os: String,
    /// Browser.
    #[serde(rename = "$browser", skip_serializing_if = "String::is_empty")]
    pub browser: String,
    /// Device.
    #[serde(rename = "$device", skip_serializing_if = "String::is_empty")]
    pub device: String,
}

/// Resume payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WsResumeData {
    /// Authorization token.
    pub token: String,
    /// Session ID.
    pub session_id: String,
    /// Last received sequence number.
    pub seq: u64,
}

/// `HELLO` payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WsHelloData {
    /// Heartbeat interval in milliseconds.
    pub heartbeat_interval: u64,
}

/// `READY` payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WsReadyData {
    /// Protocol version.
    pub version: i32,
    /// Session ID.
    pub session_id: String,
    /// Bot user.
    pub user: WsUser,
    /// `[shard_id, shard_count]`.
    pub shard: [u32; 2],
}

/// Gateway bot identity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WsUser {
    /// Bot ID.
    pub id: String,
    /// Bot username.
    pub username: String,
    /// Whether the user is a bot.
    pub bot: bool,
}

/// Gateway URL returned by the generic `/gateway` endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewayUrl {
    /// WebSocket connection URL.
    pub url: String,
}

/// Gateway connection information returned by `/gateway/bot`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebsocketAp {
    /// WebSocket URL.
    pub url: String,
    /// Number of shards.
    pub shards: u32,
    /// Session start rate-limit information.
    pub session_start_limit: SessionStartLimit,
}

/// Session start rate-limit information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionStartLimit {
    /// Total sessions allowed.
    pub total: u32,
    /// Sessions remaining.
    pub remaining: u32,
    /// Milliseconds until the limit resets.
    pub reset_after: u32,
    /// Maximum concurrent identify requests.
    pub max_concurrency: u32,
}

/// Shard configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Shard ID, starting at zero.
    pub shard_id: u32,
    /// Total number of shards.
    pub shard_count: u32,
}

impl ShardConfig {
    /// Creates a shard configuration.
    #[must_use]
    pub const fn new(shard_id: u32, shard_count: u32) -> Self {
        Self {
            shard_id,
            shard_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intents_are_bit_flags() {
        let intents = Intents::GUILDS | Intents::GUILD_MESSAGES;
        assert!(intents.contains(Intents::GUILDS));
        assert!(!intents.contains(Intents::AUDIO));
        assert_eq!(intents.bits(), 1 | (1 << 9));
    }

    #[test]
    fn event_type_serializes_as_a_string() {
        let value = serde_json::to_string(&EventType::MESSAGE_CREATE).unwrap();
        assert_eq!(value, "\"MESSAGE_CREATE\"");
        let event_type: EventType = serde_json::from_str("\"SOMETHING_NEW\"").unwrap();
        assert_eq!(event_type.as_str(), "SOMETHING_NEW");
    }

    #[test]
    fn payload_round_trips() {
        let raw = r#"{"op":0,"s":42,"t":"MESSAGE_CREATE","id":"e1","d":{"id":"m1"}}"#;
        let payload: WsPayload = serde_json::from_str(raw).unwrap();
        assert_eq!(payload.op, OpCode::DISPATCH);
        assert_eq!(payload.seq, Some(42));
        assert_eq!(payload.event_type, Some(EventType::MESSAGE_CREATE));
        assert_eq!(payload.event_id.as_deref(), Some("e1"));
    }
}
