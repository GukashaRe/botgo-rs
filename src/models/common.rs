//! Shared data transfer objects used across the OpenAPI and gateway.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

/// A timestamp encoded by the QQ Bot platform.
///
/// The platform currently returns RFC 3339 strings. The value is kept as a
/// string so that unknown or future formats are preserved losslessly.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub String);

impl Timestamp {
    /// Creates a timestamp from anything that can be converted into a string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the timestamp as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` when the timestamp is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for Timestamp {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for Timestamp {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for Timestamp {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A human-readable duration such as `"5m"` or `"1h30m"`.
///
/// This mirrors the custom `Duration` decoder in the Go SDK. The value is
/// stored as a [`std::time::Duration`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub std::time::Duration);

impl Duration {
    /// Returns the wrapped standard library duration.
    #[must_use]
    pub const fn get(self) -> std::time::Duration {
        self.0
    }
}

impl From<std::time::Duration> for Duration {
    fn from(value: std::time::Duration) -> Self {
        Self(value)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_duration(&raw)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format_duration(self.0))
    }
}

fn parse_duration(input: &str) -> Result<std::time::Duration, String> {
    let input = input.trim_matches(['"', '\'']);
    if input.is_empty() {
        return Err("duration is empty".to_owned());
    }

    let mut total = std::time::Duration::ZERO;
    let mut number = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
            continue;
        }

        let unit = match ch {
            'n' if chars.peek() == Some(&'s') => {
                chars.next();
                std::time::Duration::from_nanos(1)
            }
            'u' if chars.peek() == Some(&'s') => {
                chars.next();
                std::time::Duration::from_micros(1)
            }
            'm' if chars.peek() == Some(&'s') => {
                chars.next();
                std::time::Duration::from_millis(1)
            }
            's' => std::time::Duration::from_secs(1),
            'm' => std::time::Duration::from_secs(60),
            'h' => std::time::Duration::from_secs(60 * 60),
            _ => return Err(format!("unsupported duration unit `{ch}`")),
        };

        if number.is_empty() {
            return Err(format!("missing number before `{ch}`"));
        }
        let value: f64 = number
            .parse()
            .map_err(|_| format!("invalid duration number `{number}`"))?;
        total = total
            .checked_add(unit.mul_f64(value))
            .ok_or_else(|| "duration overflow".to_owned())?;
        number.clear();
    }

    if !number.is_empty() {
        return Err("duration is missing a unit".to_owned());
    }
    Ok(total)
}

fn format_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    if seconds > 0 && duration.subsec_nanos() == 0 {
        return format!("{seconds}s");
    }
    format!("{}ms", duration.as_millis())
}

/// A QQ user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct User {
    /// User ID.
    pub id: String,
    /// Display name.
    pub username: String,
    /// Avatar URL.
    pub avatar: String,
    /// Whether this user is a bot.
    pub bot: bool,
    /// OpenID associated with a union application.
    pub union_openid: String,
    /// Account information associated with `union_openid`.
    pub union_user_account: String,
    /// OpenID used by C2C APIs.
    pub user_openid: String,
    /// OpenID used by group member APIs.
    pub member_openid: String,
    /// Group role: `member`, `admin`, or `owner`.
    pub member_role: String,
}

/// A guild (channel server).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Guild {
    /// Guild ID.
    pub id: String,
    /// Guild name.
    pub name: String,
    /// Guild icon URL.
    pub icon: String,
    /// Owner user ID.
    pub owner_id: String,
    /// Whether the current user is the owner.
    #[serde(rename = "owner")]
    pub is_owner: bool,
    /// Number of members.
    pub member_count: i32,
    /// Maximum number of members.
    pub max_members: i64,
    /// Guild description.
    #[serde(rename = "description")]
    pub description: String,
    /// Time the current user joined the guild. Only present for `GUILD_CREATE`.
    pub joined_at: Timestamp,
    /// Channels in this guild.
    pub channels: Vec<Channel>,
    /// Game union world ID.
    pub union_world_id: String,
    /// Game union organization ID.
    pub union_org_id: String,
    /// ID of the user that performed the operation.
    pub op_user_id: String,
}

/// Channel type values understood by the platform.
pub mod channel_type {
    /// Text channel.
    pub const TEXT: i32 = 0;
    /// Voice channel.
    pub const VOICE: i32 = 2;
    /// Category.
    pub const CATEGORY: i32 = 4;
    /// Live channel.
    pub const LIVE: i32 = 10_005;
    /// Application channel.
    pub const APPLICATION: i32 = 10_006;
    /// Forum channel.
    pub const FORUM: i32 = 10_007;
}

/// A channel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Channel {
    /// Channel ID.
    pub id: String,
    /// Guild ID.
    pub guild_id: String,
    /// Channel name.
    pub name: String,
    /// Channel type. See [`channel_type`].
    #[serde(rename = "type")]
    pub channel_type: i32,
    /// Sort position.
    pub position: i64,
    /// Parent category ID.
    pub parent_id: String,
    /// Owner user ID.
    pub owner_id: String,
    /// Channel sub-type.
    pub sub_type: i32,
    /// Channel visibility.
    pub private_type: i32,
    /// Members explicitly allowed to access a private channel.
    pub private_user_ids: Vec<String>,
    /// Speaking permission.
    pub speak_permission: i32,
    /// Application ID for application channels.
    pub application_id: String,
    /// Permissions of the bot in this channel, encoded as a bit string.
    pub permissions: String,
    /// ID of the user that performed the operation.
    pub op_user_id: String,
}

/// Fields accepted when creating or updating a channel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChannelValueObject {
    /// Channel name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Channel type.
    #[serde(rename = "type", skip_serializing_if = "is_zero_i32")]
    pub channel_type: i32,
    /// Sort position.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub position: i64,
    /// Parent category ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub parent_id: String,
    /// Owner user ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub owner_id: String,
    /// Channel sub-type.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub sub_type: i32,
    /// Channel visibility.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub private_type: i32,
    /// Members to add to a private channel.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub private_user_ids: Vec<String>,
    /// Speaking permission.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub speak_permission: i32,
    /// Application ID for application channels.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub application_id: String,
    /// Permissions of the bot in this channel.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub permissions: String,
    /// ID of the user that performed the operation.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub op_user_id: String,
}

const fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

const fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

/// A guild member.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Member {
    /// Guild ID.
    pub guild_id: String,
    /// Join time.
    pub joined_at: Timestamp,
    /// Guild nickname.
    pub nick: String,
    /// User information.
    pub user: Option<User>,
    /// Role IDs.
    pub roles: Vec<String>,
    /// ID of the user that performed the operation.
    pub op_user_id: String,
}

/// Number of days of history to delete when removing a member.
pub type DeleteHistoryMsgDays = i32;
/// Alias matching the Go SDK's singular type name.
pub type DeleteHistoryMsgDay = DeleteHistoryMsgDays;

/// Do not delete any messages.
pub const NO_DELETE: DeleteHistoryMsgDays = 0;
/// Delete messages from the last three days.
pub const DELETE_THREE_DAYS: DeleteHistoryMsgDays = 3;
/// Delete messages from the last seven days.
pub const DELETE_SEVEN_DAYS: DeleteHistoryMsgDays = 7;
/// Delete messages from the last fifteen days.
pub const DELETE_FIFTEEN_DAYS: DeleteHistoryMsgDays = 15;
/// Delete messages from the last thirty days.
pub const DELETE_THIRTY_DAYS: DeleteHistoryMsgDays = 30;
/// Delete all messages.
pub const DELETE_ALL: DeleteHistoryMsgDays = -1;

/// Optional parameters for removing a guild member.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberDeleteOptions {
    /// Whether to also add the member to the guild blacklist.
    pub add_blacklist: bool,
    /// Number of days of message history to delete.
    pub delete_history_msg_days: DeleteHistoryMsgDays,
}

impl MemberDeleteOptions {
    /// Sets the blacklist flag.
    #[must_use]
    pub const fn with_add_blacklist(mut self, add: bool) -> Self {
        self.add_blacklist = add;
        self
    }

    /// Sets the number of history days to delete.
    #[must_use]
    pub const fn with_delete_history(mut self, days: DeleteHistoryMsgDays) -> Self {
        self.delete_history_msg_days = days;
        self
    }
}

/// A guild role.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Role {
    /// Role ID.
    pub id: String,
    /// Role name.
    pub name: String,
    /// ARGB color value.
    pub color: u32,
    /// Whether the role is displayed separately.
    pub hoist: u32,
    /// Number of members.
    #[serde(rename = "number")]
    pub member_count: u32,
    /// Maximum number of members.
    pub member_limit: u32,
}

/// Default role color.
pub const DEFAULT_COLOR: u32 = 4_278_245_297;

/// Editable role information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRoleInfo {
    /// Role name.
    pub name: String,
    /// ARGB color value.
    pub color: u32,
    /// Whether the role is displayed separately.
    pub hoist: u32,
}

/// A role ID.
pub type RoleId = String;

/// Roles returned by the guild role list endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GuildRoles {
    /// Guild ID.
    pub guild_id: String,
    /// Roles.
    pub roles: Vec<Role>,
    /// Maximum number of roles.
    #[serde(rename = "role_num_limit")]
    pub role_num_limit: String,
}

/// Filter for fields changed by a role update.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRoleFilter {
    /// Update the name when non-zero.
    pub name: u32,
    /// Update the color when non-zero.
    pub color: u32,
    /// Update hoist when non-zero.
    pub hoist: u32,
}

/// Role update request body.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRole {
    /// Guild ID.
    pub guild_id: String,
    /// Fields to update.
    pub filter: UpdateRoleFilter,
    /// New role values.
    pub info: Role,
}

/// Result of a role create, update, or member role operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateResult {
    /// Role ID.
    pub role_id: String,
    /// Guild ID.
    pub guild_id: String,
    /// Role.
    pub role: Option<Role>,
}

/// Extra information supplied when adding a role to a member.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberAddRoleBody {
    /// Channel that grants the role.
    pub channel: Option<Channel>,
}

/// Paging parameters for [`crate::OpenApi::guild_members`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuildMembersPager {
    /// Read members after this user ID.
    pub after: Option<String>,
    /// Page size, 1-1000.
    pub limit: Option<u32>,
}

impl GuildMembersPager {
    /// Creates an empty pager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `after` cursor.
    #[must_use]
    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Sets the page size.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(after) = &self.after {
            params.push(("after".to_owned(), after.clone()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_owned(), limit.to_string()));
        }
        params
    }
}

/// Paging parameters for [`crate::OpenApi::guild_role_members`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuildRoleMembersPager {
    /// Start index.
    pub start_index: Option<u32>,
    /// Page size.
    pub limit: Option<u32>,
}

impl GuildRoleMembersPager {
    /// Creates an empty pager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the start index.
    #[must_use]
    pub const fn start_index(mut self, start_index: u32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Sets the page size.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(start_index) = self.start_index {
            params.push(("start_index".to_owned(), start_index.to_string()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_owned(), limit.to_string()));
        }
        params
    }
}

/// Paging parameters for [`crate::OpenApi::me_guilds`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuildPager {
    /// Read guilds before this ID.
    pub before: Option<String>,
    /// Read guilds after this ID.
    pub after: Option<String>,
    /// Page size, 1-100.
    pub limit: Option<u32>,
}

impl GuildPager {
    /// Creates an empty pager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `before` cursor.
    #[must_use]
    pub fn before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }

    /// Sets the `after` cursor.
    #[must_use]
    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Sets the page size.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(after) = &self.after {
            params.push(("after".to_owned(), after.clone()));
        } else if let Some(before) = &self.before {
            params.push(("before".to_owned(), before.clone()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_owned(), limit.to_string()));
        }
        params
    }
}

/// How [`crate::OpenApi::messages`] selects messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessagePagerType {
    /// Messages around the given ID.
    Around,
    /// Messages before the given ID.
    Before,
    /// Messages after the given ID.
    After,
}

impl MessagePagerType {
    /// Returns the wire representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Around => "around",
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

/// Paging parameters for [`crate::OpenApi::messages`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessagesPager {
    /// Selection mode.
    pub kind: Option<MessagePagerType>,
    /// Anchor message ID.
    pub id: Option<String>,
    /// Page size, at most 20.
    pub limit: Option<u32>,
}

impl MessagesPager {
    /// Creates an empty pager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects messages around `id`.
    #[must_use]
    pub fn around(mut self, id: impl Into<String>) -> Self {
        self.kind = Some(MessagePagerType::Around);
        self.id = Some(id.into());
        self
    }

    /// Selects messages before `id`.
    #[must_use]
    pub fn before(mut self, id: impl Into<String>) -> Self {
        self.kind = Some(MessagePagerType::Before);
        self.id = Some(id.into());
        self
    }

    /// Selects messages after `id`.
    #[must_use]
    pub fn after(mut self, id: impl Into<String>) -> Self {
        self.kind = Some(MessagePagerType::After);
        self.id = Some(id.into());
        self
    }

    /// Sets the page size.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit".to_owned(), limit.to_string()));
        }
        if let (Some(kind), Some(id)) = (self.kind, &self.id) {
            params.push((kind.as_str().to_owned(), id.clone()));
        }
        params
    }
}

/// Paging parameters for message reaction users.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageReactionPager {
    /// Opaque pagination cursor.
    pub cookie: Option<String>,
    /// Page size.
    pub limit: Option<u32>,
}

impl MessageReactionPager {
    /// Creates an empty pager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pagination cursor.
    #[must_use]
    pub fn cookie(mut self, cookie: impl Into<String>) -> Self {
        self.cookie = Some(cookie.into());
        self
    }

    /// Sets the page size.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(cookie) = &self.cookie {
            params.push(("cookie".to_owned(), cookie.clone()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_owned(), limit.to_string()));
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_deserializes_go_style_values() {
        let duration: Duration = serde_json::from_str("\"1h30m\"").unwrap();
        assert_eq!(duration.get(), std::time::Duration::from_secs(5400));

        let duration: Duration = serde_json::from_str("\"250ms\"").unwrap();
        assert_eq!(duration.get(), std::time::Duration::from_millis(250));
    }

    #[test]
    fn pager_prioritises_after_over_before() {
        let pager = GuildPager::new().before("1").after("2").limit(10);
        assert_eq!(
            pager.query_params(),
            vec![
                ("after".to_owned(), "2".to_owned()),
                ("limit".to_owned(), "10".to_owned()),
            ]
        );
    }
}
