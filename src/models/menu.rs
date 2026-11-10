//! Global custom menu models.

use serde::{Deserialize, Serialize};

/// A global C2C custom menu.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Menu {
    /// Menu items, at most 10.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<MenuItem>,
}

/// A top-level menu item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuItem {
    /// Button label, at most 10 characters.
    pub name: String,
    /// `switch`, `send_message`, `link`, or `menu`.
    #[serde(rename = "type")]
    pub item_type: String,
    /// Child items, only for `type = menu`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sub_menu_items: Vec<SubMenuItem>,
    /// Text inserted into the input box for `send_message`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub send_message: String,
    /// HTTPS link for `link`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub link: String,
    /// Switch configuration for `switch`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch: Option<MenuSwitch>,
}

/// A second-level menu item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubMenuItem {
    /// Button label, at most 14 characters.
    pub name: String,
    /// `send_message` or `link`.
    #[serde(rename = "type")]
    pub item_type: String,
    /// Text inserted into the input box.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub send_message: String,
    /// HTTPS link.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub link: String,
}

/// A menu toggle switch.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuSwitch {
    /// Unique switch ID.
    pub switch_id: String,
    /// Default state.
    pub default: bool,
}

/// Current menu response.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuResponse {
    /// Current menu version.
    pub version: i64,
    /// Current menu, absent when no menu has been configured.
    pub menu: Option<Menu>,
}

/// Request body for replacing the global menu.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuUpdate {
    /// New menu configuration.
    pub menu: Menu,
}

/// Result of a menu update.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuUpdateResult {
    /// New menu version.
    pub version: i64,
}
