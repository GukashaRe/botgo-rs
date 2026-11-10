//! Announcement, permission, audio, direct-message, and gateway models.

use serde::{Deserialize, Serialize};

/// A guild announcement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Announces {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Message used to create the announcement.
    pub message_id: String,
    /// 0 member announcement, 1 welcome announcement.
    pub announces_type: u32,
    /// Recommended channels.
    pub recommend_channels: Vec<RecommendChannel>,
}

/// Request body for a channel announcement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelAnnouncesToCreate {
    /// Message ID.
    pub message_id: String,
}

/// Request body for a guild announcement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GuildAnnouncesToCreate {
    /// Channel ID.
    pub channel_id: String,
    /// Message ID.
    pub message_id: String,
    /// 0 member announcement, 1 welcome announcement.
    pub announces_type: u32,
    /// Recommended channels.
    pub recommend_channels: Vec<RecommendChannel>,
}

/// A recommended channel in an announcement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RecommendChannel {
    /// Channel ID.
    pub channel_id: String,
    /// Recommendation text.
    pub introduce: String,
}

/// An API permission list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiPermissions {
    /// Available APIs.
    #[serde(rename = "apis")]
    pub apis: Vec<ApiPermission>,
}

/// A single API permission.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiPermission {
    /// API path.
    pub path: String,
    /// HTTP method.
    pub method: String,
    /// Description.
    pub desc: String,
    /// Authorization status; 1 means authorized.
    pub auth_status: i32,
}

/// Identifies an API permission.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiPermissionDemandIdentify {
    /// API path.
    pub path: String,
    /// HTTP method.
    pub method: String,
}

/// An API permission demand.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiPermissionDemand {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// API identifier.
    pub api_identify: Option<ApiPermissionDemandIdentify>,
    /// Title.
    pub title: String,
    /// Description.
    pub desc: String,
}

/// Request body for creating an API permission demand.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiPermissionDemandToCreate {
    /// Channel ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub channel_id: String,
    /// API identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_identify: Option<ApiPermissionDemandIdentify>,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub desc: String,
}

/// Audio playback status.
pub mod audio_status {
    /// Start playback.
    pub const START: u32 = 0;
    /// Pause playback.
    pub const PAUSE: u32 = 1;
    /// Resume playback.
    pub const RESUME: u32 = 2;
    /// Stop playback.
    pub const STOP: u32 = 3;
}

/// Audio control request and response.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioControl {
    /// Audio URL.
    #[serde(rename = "audio_url")]
    pub url: String,
    /// Text shown with the audio.
    pub text: String,
    /// Playback status.
    pub status: u32,
}

/// Audio gateway event payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioAction {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Audio URL.
    #[serde(rename = "audio_url")]
    pub url: String,
    /// Text.
    pub text: String,
}

/// Channel permissions for a user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChannelPermissions {
    /// Channel ID.
    pub channel_id: String,
    /// User ID.
    pub user_id: String,
    /// Permission bit string.
    pub permissions: String,
}

/// Channel permissions for a role.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChannelRolesPermissions {
    /// Channel ID.
    pub channel_id: String,
    /// Role ID.
    pub role_id: String,
    /// Permission bit string.
    pub permissions: String,
}

/// Update to a channel permission bit string.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateChannelPermissions {
    /// Bits to add.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub add: String,
    /// Bits to remove.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remove: String,
}

/// A direct-message channel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DirectMessage {
    /// Guild ID of the direct-message channel.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Creation time.
    pub create_time: String,
}

/// Request to create a direct-message channel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectMessageToCreate {
    /// Source guild ID.
    pub source_guild_id: String,
    /// Recipient user ID.
    pub recipient_id: String,
}

/// A guild schedule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Schedule {
    /// Schedule ID.
    pub id: String,
    /// Name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Start timestamp.
    pub start_timestamp: String,
    /// End timestamp.
    pub end_timestamp: String,
    /// Channel to jump to.
    pub jump_channel_id: String,
    /// Reminder type.
    pub remind_type: String,
    /// Creator.
    pub creator: Option<super::common::Member>,
}

/// Wrapper used when creating or updating a schedule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleWrapper {
    /// Schedule.
    pub schedule: Option<Schedule>,
}

/// A pinned-message list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PinsMessage {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Pinned message IDs.
    pub message_ids: Vec<String>,
}

/// Request body for muting a guild or members.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateGuildMute {
    /// Mute end timestamp in seconds.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mute_end_timestamp: String,
    /// Mute duration in seconds.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mute_seconds: String,
    /// User IDs to mute. Omit for an `@everyone` mute.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub user_ids: Vec<String>,
}

/// Result of a bulk mute operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateGuildMuteResponse {
    /// Successfully muted user IDs.
    pub user_ids: Vec<String>,
}

/// Identity used when creating an HTTP callback session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpIdentity {
    /// Subscribed intents.
    pub intents: super::gateway::Intents,
    /// `[shard_id, shard_count]`.
    pub shards: [u32; 2],
    /// Callback URL.
    pub callback_url: String,
}

/// Response from creating an HTTP callback session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpReady {
    /// Protocol version.
    pub version: i32,
    /// Session ID.
    pub session_id: String,
    /// Bot information.
    pub bot: HttpBot,
    /// `[shard_id, shard_count]`.
    pub shard: [u32; 2],
}

/// Bot identity in an HTTP ready payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpBot {
    /// Bot ID.
    pub id: String,
    /// Bot username.
    pub username: String,
}

/// An HTTP callback session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpSession {
    /// Application ID.
    pub app_id: i64,
    /// Session ID.
    pub session_id: String,
    /// Callback URL.
    pub callback_url: String,
    /// Environment.
    pub env: String,
    /// Subscribed intents.
    pub intents: i64,
    /// Last heartbeat time.
    pub last_heartbeat_time: String,
    /// Session state.
    pub state: String,
    /// Shard configuration.
    pub shards: [i64; 2],
}

/// Webhook URL verification request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookValidationRequest {
    /// Plain token.
    pub plain_token: String,
    /// Event timestamp.
    pub event_ts: String,
}

/// Webhook URL verification response.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebhookValidationResponse {
    /// Echoed plain token.
    pub plain_token: String,
    /// Ed25519 signature.
    pub signature: String,
    /// Data format version.
    pub data_version: String,
}

/// HTTP callback acknowledgement payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookAck {
    /// Opcode.
    pub op: i32,
    /// Acknowledgement data.
    pub d: u32,
}
