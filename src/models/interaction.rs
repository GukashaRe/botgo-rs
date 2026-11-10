//! Interaction, emoji, forum, subscription, and C2C event models.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

/// An emoji reference.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Emoji {
    /// Emoji ID.
    pub id: String,
    /// Emoji type. Type 1 is a system emoji.
    #[serde(rename = "type")]
    pub emoji_type: i32,
}

/// Target of a reaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReactionTarget {
    /// Target ID.
    pub id: String,
    /// Target type: 0 message, 1 feed, 2 comment, 3 reply.
    #[serde(rename = "type")]
    pub target_type: i32,
}

/// A reaction add or remove event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageReaction {
    /// User ID.
    pub user_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Guild ID.
    pub guild_id: String,
    /// Reaction target.
    pub target: ReactionTarget,
    /// Emoji.
    pub emoji: Emoji,
}

/// An interaction event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Interaction {
    /// Interaction ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Application ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub application_id: String,
    /// Interaction type.
    #[serde(rename = "type", skip_serializing_if = "is_zero_u32")]
    pub interaction_type: u32,
    /// Interaction data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<InteractionData>,
    /// Guild ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guild_id: String,
    /// Channel ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub channel_id: String,
    /// Version.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub version: u32,
    /// Group OpenID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub group_openid: String,
    /// 0 guild, 1 group, 2 C2C.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub chat_type: u32,
    /// Scene: `c2c`, `group`, or `guild`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub scene: String,
    /// User OpenID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub user_openid: String,
    /// Timestamp.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub timestamp: String,
    /// Group member OpenID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub group_member_openid: String,
}

const fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

/// Interaction type values.
pub mod interaction_type {
    /// Ping.
    pub const PING: u32 = 1;
    /// Command.
    pub const COMMAND: u32 = 2;
    /// Inline keyboard button click.
    pub const INLINE_KEYBOARD: u32 = 11;
    /// C2C shortcut-menu click.
    pub const CALLBACK_COMMAND: u32 = 12;
    /// Message feedback.
    pub const MESSAGE_FEEDBACK: u32 = 13;
    /// Clear conversation.
    pub const CLEAR_SESSION: u32 = 14;
    /// Enter or leave a story.
    pub const IN_OUT_STORY: u32 = 15;
    /// Switch model.
    pub const SWITCH_MODEL: u32 = 16;
    /// User authorization.
    pub const USER_AUTHORIZE: u32 = 18;
    /// Group authorization.
    pub const GROUP_AUTHORIZE: u32 = 19;
    /// Group authorization status change.
    pub const GROUP_AUTHORIZE_STATUS: u32 = 20;
}

/// Interaction payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InteractionData {
    /// Title or name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Data type.
    #[serde(rename = "type", skip_serializing_if = "is_zero_u32")]
    pub data_type: u32,
    /// Type-specific resolved payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<serde_json::Value>,
}

/// Interaction data type values.
pub mod interaction_data_type {
    /// Chat-box search.
    pub const CHAT_SEARCH: u32 = 9;
    /// Inline keyboard click.
    pub const INLINE_KEYBOARD_CLICK: u32 = 11;
    /// C2C menu click.
    pub const CALLBACK_COMMAND_CLICK: u32 = 12;
    /// Agent message feedback.
    pub const MESSAGE_FEEDBACK_CLICK: u32 = 13;
    /// Clear-session button.
    pub const CLEAR_SESSION_CLICK: u32 = 14;
    /// Enter or leave a story.
    pub const IN_OUT_STORY: u32 = 15;
    /// Switch model.
    pub const SWITCH_MODEL: u32 = 16;
    /// User authorization.
    pub const USER_AUTHORIZE: u32 = 18;
    /// Group authorization.
    pub const GROUP_AUTHORIZE: u32 = 19;
    /// Group authorization status change.
    pub const GROUP_AUTHORIZE_STATUS: u32 = 20;
}

/// Resolved payload for inline search.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchInputResolved {
    /// Search keyword.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub keyword: String,
}

/// Inline search response.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchResponse {
    /// Result layouts.
    pub layouts: Vec<SearchLayout>,
}

/// A search result layout.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchLayout {
    /// Layout type.
    #[serde(rename = "layout_type", alias = "LayoutType")]
    pub layout_type: u32,
    /// Click action type.
    #[serde(rename = "action_type", alias = "ActionType")]
    pub action_type: u32,
    /// Title.
    #[serde(alias = "Title")]
    pub title: String,
    /// Records.
    #[serde(alias = "Records")]
    pub records: Vec<SearchRecord>,
}

/// A single inline search result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchRecord {
    /// Cover image URL.
    pub cover: String,
    /// Title.
    pub title: String,
    /// Tips.
    pub tips: String,
    /// Target URL.
    pub url: String,
}

/// Generic resolved interaction payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Resolved {
    /// Keyword.
    pub keyword: String,
    /// User ID.
    pub user_id: String,
    /// Request.
    pub request: String,
    /// Message ID.
    pub message_id: String,
    /// Member nickname.
    pub member_nick: String,
    /// Button data.
    pub button_data: String,
    /// Button ID.
    pub button_id: String,
    /// Feature ID.
    pub feature_id: String,
    /// Feedback option, `LIKE` or `UNLIKE`.
    pub feedback_opt: String,
    /// Whether the feedback option is selected.
    pub checked: i32,
    /// Interaction action, such as `ENTER_STORY` or `QUIT_STORY`.
    pub action: String,
    /// Message scene information for feedback interactions.
    pub message_scene: Option<InteractionMessageScene>,
    /// Authorization data for user or group authorization interactions.
    pub authorize_data: Option<AuthorizeData>,
}

/// Message scene information attached to feedback interactions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InteractionMessageScene {
    /// Key-value extension entries.
    pub ext: Vec<String>,
}

/// Authorization data for user or group authorization interactions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthorizeData {
    /// Authorization scene: `setting` or `dialog`.
    pub opt_scene: String,
    /// Scope: `c2c_push` or `group_push`.
    pub scope: String,
}

/// A forum thread event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Thread {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Author ID.
    pub author_id: String,
    /// Thread information.
    pub thread_info: ThreadInfo,
}

/// Forum thread information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThreadInfo {
    /// Thread ID.
    pub thread_id: String,
    /// Title.
    pub title: String,
    /// Content.
    pub content: String,
    /// Creation time.
    pub date_time: String,
}

/// A forum post event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Post {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Author ID.
    pub author_id: String,
    /// Post information.
    pub post_info: PostInfo,
}

/// Forum post information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PostInfo {
    /// Thread ID.
    pub thread_id: String,
    /// Post ID.
    pub post_id: String,
    /// Content.
    pub content: String,
    /// Creation time.
    pub date_time: String,
}

/// A forum reply event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reply {
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Author ID.
    pub author_id: String,
    /// Reply information.
    pub reply_info: ReplyInfo,
}

/// Forum reply information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplyInfo {
    /// Thread ID.
    pub thread_id: String,
    /// Post ID.
    pub post_id: String,
    /// Reply ID.
    pub reply_id: String,
    /// Content.
    pub content: String,
    /// Creation time.
    pub date_time: String,
}

/// Forum publish audit result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ForumAuditResult {
    /// Audit task ID.
    pub task_id: String,
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Author ID.
    pub author_id: String,
    /// Thread ID.
    pub thread_id: String,
    /// Post ID.
    pub post_id: String,
    /// Reply ID.
    pub reply_id: String,
    /// Publish type.
    #[serde(rename = "type")]
    pub publish_type: u32,
    /// Audit result.
    pub result: u32,
    /// Error message.
    pub err_msg: String,
    /// Creation time.
    pub date_time: String,
}

/// Subscription template authorization status.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubscribeMessageStatusData {
    /// Group OpenID, present for group subscriptions.
    pub group_openid: String,
    /// User OpenID, present for user subscriptions.
    pub openid: String,
    /// Authorization results.
    pub result: Vec<SubscribeMsgTemplateResult>,
}

/// A single subscription authorization result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubscribeMsgTemplateResult {
    /// Official template ID.
    pub template_id: i32,
    /// Custom template ID.
    pub custom_template_id: String,
    /// 1 allowed, 2 rejected.
    pub op: u32,
    /// Subscription ID.
    pub subscribe_id: String,
    /// Last update timestamp.
    pub update_ts: u64,
}

/// A C2C friend add or remove event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct C2cFriendData {
    /// User OpenID.
    pub openid: String,
    /// Event timestamp.
    pub timestamp: i32,
    /// Nickname, when provided by the event.
    pub nick: String,
    /// Avatar URL, when provided by the event.
    pub avatar: String,
}

/// User entered an AIO session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnterAio {
    /// User OpenID.
    pub user_openid: String,
    /// Source of the entry.
    pub from_source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_omits_empty_fields() {
        let interaction = Interaction {
            application_id: "app".to_owned(),
            interaction_type: 2,
            version: 1,
            ..Interaction::default()
        };
        let json = serde_json::to_value(interaction).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "application_id": "app",
                "type": 2,
                "version": 1
            })
        );
    }

    #[test]
    fn search_layout_accepts_legacy_go_keys() {
        let layout: SearchLayout = serde_json::from_value(serde_json::json!({
            "LayoutType": 0,
            "ActionType": 0,
            "Title": "result",
            "Records": []
        }))
        .unwrap();
        assert_eq!(layout.title, "result");
    }
}
