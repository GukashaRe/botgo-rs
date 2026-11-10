//! Message models, from both the OpenAPI and the gateway.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

use super::common::{Member, Timestamp, User};
use super::keyboard::MessageKeyboard;
use super::upload::MessageExtInfo;

/// Message payload type values.
pub mod message_type {
    /// Plain text.
    pub const TEXT: i32 = 0;
    /// Markdown.
    pub const MARKDOWN: i32 = 2;
    /// Ark template.
    pub const ARK: i32 = 3;
    /// Embed.
    pub const EMBED: i32 = 4;
    /// AT message.
    pub const AT: i32 = 5;
    /// Input notification.
    pub const INPUT_NOTIFY: i32 = 6;
    /// Rich media.
    pub const RICH_MEDIA: i32 = 7;
    /// Image-text card.
    pub const CARD: i32 = 8;
    /// Parallel messages.
    pub const PARALLEL: i32 = 101;
    /// Chat history.
    pub const CHAT_RECORD: i32 = 102;
    /// Quoted message.
    pub const QUOTE: i32 = 103;
}

/// A message returned by the platform.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Message {
    /// Message ID.
    pub id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Guild ID.
    pub guild_id: String,
    /// Group ID, for group messages.
    pub group_id: String,
    /// Group OpenID used by API v2 group messages.
    pub group_openid: String,
    /// Text content.
    pub content: String,
    /// Message content type.
    pub message_type: i32,
    /// Send time.
    pub timestamp: Timestamp,
    /// Edit time.
    pub edited_timestamp: Timestamp,
    /// Whether `@everyone` is mentioned.
    pub mention_everyone: bool,
    /// Author.
    pub author: Option<User>,
    /// Author's member information, when available.
    pub member: Option<Member>,
    /// Attachments.
    pub attachments: Vec<MessageAttachment>,
    /// Embeds.
    pub embeds: Vec<Embed>,
    /// Mentioned users.
    pub mentions: Vec<User>,
    /// Ark payload.
    pub ark: Option<Ark>,
    /// Structured card data returned by group and C2C events.
    pub ark_data: Option<ArkData>,
    /// Structured message elements, including quoted messages.
    pub msg_elements: Vec<MsgElement>,
    /// Whether this is a direct message.
    pub direct_message: bool,
    /// Per-channel sequence number.
    pub seq_in_channel: String,
    /// Referenced message.
    pub message_reference: Option<MessageReference>,
    /// Guild that started a direct message.
    pub src_guild_id: String,
    /// File information returned after uploading rich media.
    pub file_info: Option<String>,
    /// Number of seconds the uploaded file remains valid.
    pub ttl: Option<u32>,
    /// Message scene.
    pub message_scene: Option<MessageScene>,
    /// Extension information returned by message-send endpoints.
    pub ext_info: Option<MessageExtInfo>,
}

/// An image-text card message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageCard {
    /// Card type; currently `tuwen`.
    #[serde(rename = "type")]
    pub card_type: String,
    /// Card content.
    pub content: MessageCardContent,
}

/// Content of an image-text card.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageCardContent {
    /// Title, 1-50 characters.
    pub title: String,
    /// Description, 1-200 characters.
    pub description: String,
    /// HTTPS target URL.
    pub url: String,
    /// HTTPS image URL.
    pub pic_url: String,
}

/// Structured card data returned by group and C2C events.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArkData {
    /// Prompt text from the card.
    pub prompt: String,
    /// Card type, for example `tuwen`, `miniapp`, or `feed`.
    pub ark_type: String,
    /// Human-readable card type name.
    pub ark_name: String,
    /// Card fields such as `title`, `desc`, and `jump_url`.
    pub fields: serde_json::Map<String, serde_json::Value>,
}

/// A structured element in a message, including quoted content.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MsgElement {
    /// Reference index of this element.
    pub msg_idx: String,
    /// Author of the quoted message.
    pub author: Option<User>,
    /// Message content type.
    pub message_type: i32,
    /// Text content.
    pub content: String,
    /// Attachments carried by this element.
    pub attachments: Vec<MessageAttachment>,
    /// Structured card data.
    pub ark_data: Option<ArkData>,
    /// Nested elements.
    pub msg_elements: Vec<Self>,
}

/// An embed object.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Embed {
    /// Title.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Prompt shown in the message list.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prompt: String,
    /// Thumbnail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<MessageEmbedThumbnail>,
    /// Fields.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<EmbedField>,
}

/// Thumbnail used by an embed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageEmbedThumbnail {
    /// Image URL.
    pub url: String,
}

/// A field in an embed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbedField {
    /// Field name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Field value.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub value: String,
}

/// A message attachment.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageAttachment {
    /// Download URL.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// File name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub filename: String,
    /// Height in pixels.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub height: i32,
    /// Size in bytes.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub size: i32,
    /// Width in pixels.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub width: i32,
    /// MIME type, such as `image/png` or `video/mp4`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content_type: String,
    /// WAV URL converted from a SILK voice message.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub voice_wav_url: String,
    /// Automatic speech recognition text for a voice message.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub asr_refer_text: String,
}

const fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

const fn is_false(value: &bool) -> bool {
    !*value
}

const fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

const fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

/// The scene in which a message was sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageScene {
    /// Source, for example `realtime_voice` or `ai_search`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub source: String,
    /// Opaque callback data.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub callback_data: String,
    /// Key-value extension entries such as `msg_idx` and `ref_msg_idx`.
    pub ext: Vec<String>,
}

/// Users that reacted to a message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageReactionUsers {
    /// Users.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<User>,
    /// Next-page cursor.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cookie: String,
    /// Whether this is the last page.
    #[serde(skip_serializing_if = "is_false")]
    pub is_end: bool,
}

/// Message send type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendType {
    /// Text-like message sent to `.../messages`.
    Text,
    /// Rich media sent to `.../files`.
    RichMedia,
}

/// A message accepted by the group and C2C endpoints.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiMessage {
    /// Text, markdown, ark, embed, or keyboard message.
    Text(MessageToCreate),
    /// Rich media message.
    RichMedia(RichMediaMessage),
}

impl ApiMessage {
    /// Returns the event ID this message replies to, if any.
    #[must_use]
    pub fn event_id(&self) -> &str {
        match self {
            Self::Text(message) => &message.event_id,
            Self::RichMedia(message) => &message.event_id,
        }
    }

    /// Returns the endpoint family this message must use.
    #[must_use]
    pub const fn send_type(&self) -> SendType {
        match self {
            Self::Text(_) => SendType::Text,
            Self::RichMedia(_) => SendType::RichMedia,
        }
    }
}

impl From<MessageToCreate> for ApiMessage {
    fn from(value: MessageToCreate) -> Self {
        Self::Text(value)
    }
}

impl From<RichMediaMessage> for ApiMessage {
    fn from(value: RichMediaMessage) -> Self {
        Self::RichMedia(value)
    }
}

/// A rich media message.
///
/// Prefer uploading the file first and then sending a [`MessageToCreate`] with
/// a media payload; direct rich media messages consume active-message quota and
/// cannot be reused.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RichMediaMessage {
    /// Deprecated event ID this message replies to.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub event_id: String,
    /// File type: 1 image, 2 video, 3 voice.
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub file_type: u64,
    /// HTTP(S) URL of the file.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// Send immediately when `true`, otherwise only upload the file.
    #[serde(skip_serializing_if = "is_false")]
    pub srv_send_msg: bool,
    /// Optional text content.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Reply sequence number.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub msg_seq: i64,
}

const fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

/// Message creation request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageToCreate {
    /// Text content.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Message type. See [`message_type`].
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub msg_type: i32,
    /// Embed payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<Embed>,
    /// Ark payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark: Option<Ark>,
    /// Base64 image.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image: String,
    /// ID of the message being replied to.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub msg_id: String,
    /// Message reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    /// Markdown payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<Markdown>,
    /// Keyboard payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<MessageKeyboard>,
    /// ID of the event being replied to.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub event_id: String,
    /// Millisecond timestamp.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub timestamp: i64,
    /// Reply sequence number.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub msg_seq: u32,
    /// Subscription ID for subscription messages.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub subscribe_id: String,
    /// Typing indicator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_notify: Option<InputNotify>,
    /// Rich media file information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaInfo>,
    /// Prompt keyboard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_keyboard: Option<PromptKeyboard>,
    /// Action buttons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_button: Option<ActionButton>,
    /// Streaming message metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<Stream>,
    /// Feature flag controlling message delivery.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub feature_id: u32,
    /// Image-text card, used when `msg_type = 8`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<MessageCard>,
    /// Mark this as an interaction recall message for C2C.
    #[serde(skip_serializing_if = "is_false")]
    pub is_wakeup: bool,
}

/// Streaming message metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stream {
    /// 1 generating, 10 body complete, 11 guidance generating, 20 guidance complete.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub state: i32,
    /// Stream ID; omitted on the first chunk.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Chunk index, starting at 1.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub index: i32,
    /// Regenerate the stream.
    #[serde(skip_serializing_if = "is_false")]
    pub reset: bool,
}

/// Keyboard shown in the interaction area.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptKeyboard {
    /// Keyboard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<MessageKeyboard>,
}

/// Action buttons attached to a message.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionButton {
    /// Template ID. Deprecated.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub template_id: i32,
    /// Opaque callback data, at most 128 bytes.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub callback_data: String,
    /// Feedback (like/dislike) button.
    #[serde(skip_serializing_if = "is_false")]
    pub feedback: bool,
    /// Text-to-speech button.
    #[serde(skip_serializing_if = "is_false")]
    pub tts: bool,
    /// Regenerate button.
    #[serde(skip_serializing_if = "is_false")]
    pub re_generate: bool,
    /// Stop-generation button.
    #[serde(skip_serializing_if = "is_false")]
    pub stop_generate: bool,
}

/// Reference to another message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageReference {
    /// Referenced message ID.
    pub message_id: String,
    /// Whether to ignore errors while fetching the referenced message.
    pub ignore_get_message_error: bool,
}

/// Markdown message payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Markdown {
    /// Official template ID.
    pub template_id: u32,
    /// Custom template ID.
    pub custom_template_id: String,
    /// Template parameters.
    pub params: Vec<MarkdownParams>,
    /// Raw markdown content.
    pub content: String,
    /// Style options.
    pub style: Option<MarkdownStyle>,
    /// Guidance message shown while processing.
    pub process_msg: String,
    /// Verify image resource conversion before sending.
    #[serde(default, skip_serializing_if = "is_false")]
    pub force_verify_image_resource: bool,
}

/// Markdown style options.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkdownStyle {
    /// Body font size: `small`, `middle`, or `large`.
    pub main_font_size: String,
    /// Layout, for example `hide_avatar_and_center`.
    pub layout: String,
}

/// A markdown template parameter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownParams {
    /// Parameter key.
    pub key: String,
    /// Parameter values.
    pub values: Vec<String>,
}

/// Settings guidance message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingGuideToCreate {
    /// Text content; may mention users.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Guideline target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setting_guide: Option<SettingGuide>,
}

/// Target guild for a settings guidance message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingGuide {
    /// Guild ID.
    pub guild_id: String,
}

/// Typing indicator payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputNotify {
    /// 1 to show typing, 2 to cancel.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub input_type: i32,
    /// How long the indicator remains visible.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub input_second: i32,
}

/// Rich media file information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaInfo {
    /// File information returned by the upload endpoint.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub file_info: String,
}

/// Ark message payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ark {
    /// Ark template ID.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub template_id: i32,
    /// Key-value entries.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kv: Vec<ArkKv>,
}

/// Wrapper for an Ark template message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageArk {
    /// Ark payload.
    pub ark: Option<Ark>,
}

/// An Ark key-value entry.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArkKv {
    /// Key.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub key: String,
    /// Value.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub value: String,
    /// Nested objects.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub obj: Vec<ArkObj>,
}

/// An Ark object.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArkObj {
    /// Object entries.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub obj_kv: Vec<ArkObjKv>,
}

/// An Ark object key-value entry.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArkObjKv {
    /// Key.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub key: String,
    /// Value.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub value: String,
}

/// A deleted message event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageDelete {
    /// Deleted message.
    pub message: Message,
    /// User that deleted the message.
    pub op_user: User,
}

/// A message audit event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageAudit {
    /// Audit ID.
    pub audit_id: String,
    /// Message ID.
    pub message_id: String,
    /// Guild ID.
    pub guild_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Audit time.
    pub audit_time: String,
    /// Creation time.
    pub create_time: String,
    /// Per-channel sequence.
    pub seq_in_channel: String,
}

/// Message frequency settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageSetting {
    /// Whether creating direct messages is disabled.
    pub disable_create_dm: bool,
    /// Whether pushing messages is disabled.
    pub disable_push_msg: bool,
    /// Channel IDs affected by the setting.
    pub channel_ids: Vec<String>,
    /// Maximum number of pushed messages per channel.
    pub channel_push_max_num: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_create_skips_empty_fields() {
        let value = MessageToCreate {
            content: "hello".to_owned(),
            ..MessageToCreate::default()
        };
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json, serde_json::json!({ "content": "hello" }));
    }

    #[test]
    fn api_message_serializes_transparently() {
        let message = ApiMessage::from(MessageToCreate {
            content: "hi".to_owned(),
            ..MessageToCreate::default()
        });
        assert_eq!(
            serde_json::to_value(message).unwrap(),
            serde_json::json!({ "content": "hi" })
        );
    }

    #[test]
    fn card_and_wakeup_fields_serialize() {
        let message = MessageToCreate {
            msg_type: message_type::CARD,
            card: Some(MessageCard {
                card_type: "tuwen".to_owned(),
                content: MessageCardContent {
                    title: "title".to_owned(),
                    description: "description".to_owned(),
                    url: "https://example.com".to_owned(),
                    pic_url: "https://example.com/p.png".to_owned(),
                },
            }),
            is_wakeup: true,
            ..MessageToCreate::default()
        };
        let json = serde_json::to_value(message).unwrap();
        assert_eq!(json["msg_type"], 8);
        assert_eq!(json["card"]["type"], "tuwen");
        assert_eq!(json["is_wakeup"], serde_json::json!(true));
    }
}
