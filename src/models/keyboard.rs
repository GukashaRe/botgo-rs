//! Message keyboard components.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

/// Button action types.
pub mod action_type {
    /// Open a URL or mini-program.
    pub const URL: u32 = 0;
    /// Send an interaction callback.
    pub const CALLBACK: u32 = 1;
    /// Mention the bot.
    pub const AT_BOT: u32 = 2;
    /// Native QQ jump link.
    pub const MQQ_API: u32 = 3;
    /// Subscription button.
    pub const SUBSCRIBE: u32 = 4;
}

/// Button permission types.
pub mod permission_type {
    /// Only explicitly listed user IDs may interact.
    pub const SPECIFY_USER_IDS: u32 = 0;
    /// Only guild managers may interact.
    pub const MANAGER: u32 = 1;
    /// Everyone may interact.
    pub const ALL: u32 = 2;
    /// Only members of the listed role IDs may interact.
    pub const SPECIFY_ROLE_IDS: u32 = 3;
}

/// A keyboard component.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageKeyboard {
    /// Keyboard template ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Custom keyboard content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<CustomKeyboard>,
}

/// A custom keyboard.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomKeyboard {
    /// Rows of buttons.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Row>,
    /// Style options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<KeyboardStyle>,
}

/// Keyboard style.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardStyle {
    /// Font size.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub font_size: String,
}

/// A row of buttons.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Row {
    /// Buttons.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buttons: Vec<Button>,
}

/// A keyboard button.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Button {
    /// Button ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Render data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_data: Option<RenderData>,
    /// Click action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    /// Group ID. Buttons in the same group are disabled together.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub group_id: String,
}

/// Button rendering data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderData {
    /// Label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Label after the button has been clicked.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub visited_label: String,
    /// 0 gray outline, 1 blue outline, 3 white background with red text,
    /// 4 blue background with white text.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub style: i32,
}

const fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

/// A button click action.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Action {
    /// Action type. See [`action_type`].
    #[serde(rename = "type", skip_serializing_if = "is_zero_u32")]
    pub action_type: u32,
    /// Permission required to click.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Permission>,
    /// Maximum number of clicks.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub click_limit: u32,
    /// Action-specific data.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub data: String,
    /// Whether the action triggers an enter event.
    pub enter: bool,
    /// Whether to show the channel selector after mentioning the bot.
    #[serde(skip_serializing_if = "is_false")]
    pub at_bot_show_channel_list: bool,
    /// Tooltip shown when the client version is too old.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub unsupport_tips: String,
    /// Whether a command button should quote the original message.
    #[serde(skip_serializing_if = "is_false")]
    pub reply: bool,
    /// Command-button behavior; `1` opens the image picker on mobile.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub anchor: i32,
    /// Subscription data, used when the action type is [`action_type::SUBSCRIBE`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe_data: Option<SubscribeData>,
    /// Optional confirmation modal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modal: Option<Modal>,
}

const fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

const fn is_false(value: &bool) -> bool {
    !*value
}

/// A confirmation modal.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Modal {
    /// Confirmation text, at most 40 characters and without URLs.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Confirm button label, at most 4 characters.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub confirm_text: String,
    /// Cancel button label, at most 4 characters.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cancel_text: String,
}

/// Permission required to click a button.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Permission {
    /// Permission type. See [`permission_type`].
    #[serde(rename = "type", skip_serializing_if = "is_zero_u32")]
    pub permission_type: u32,
    /// Role IDs allowed to click.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub specify_role_ids: Vec<String>,
    /// User IDs allowed to click.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub specify_user_ids: Vec<String>,
}

/// Template ID, either an official or a custom template.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateId {
    /// Official template ID.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub template_id: u32,
    /// Custom template ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub custom_template_id: String,
}

/// Subscription button data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubscribeData {
    /// Subscription templates.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub template_ids: Vec<TemplateId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_serializes_nested_components() {
        let keyboard = MessageKeyboard {
            content: Some(CustomKeyboard {
                rows: vec![Row {
                    buttons: vec![Button {
                        id: "b1".to_owned(),
                        render_data: Some(RenderData {
                            label: "Open".to_owned(),
                            ..RenderData::default()
                        }),
                        action: Some(Action {
                            action_type: action_type::URL,
                            data: "https://example.com".to_owned(),
                            ..Action::default()
                        }),
                        ..Button::default()
                    }],
                }],
                ..CustomKeyboard::default()
            }),
            ..MessageKeyboard::default()
        };

        let json = serde_json::to_value(keyboard).unwrap();
        assert_eq!(json["content"]["rows"][0]["buttons"][0]["id"], "b1");
        assert_eq!(
            json["content"]["rows"][0]["buttons"][0]["action"]["data"],
            "https://example.com"
        );
    }
}
