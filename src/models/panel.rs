//! Command panel models.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

/// A command panel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Panel {
    /// Panel items, at most 20.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<PanelItem>,
    /// Developer note, not shown to users.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remark: String,
    /// Current version.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub version: i64,
}

const fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

/// A command or link item in a panel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelItem {
    /// Item name, at most 14 characters.
    pub name: String,
    /// Description, at most 30 characters.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub desc: String,
    /// `command` or `link`.
    #[serde(rename = "type")]
    pub item_type: String,
    /// Whether only administrators can use the item.
    #[serde(skip_serializing_if = "is_false")]
    pub only_admin: bool,
    /// HTTPS link for `type = link`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub link: String,
}

const fn is_false(value: &bool) -> bool {
    !*value
}

/// A panel record returned by the list endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelRecord {
    /// Panel ID.
    pub panel_id: String,
    /// `c2c`, `group`, `channel`, or `dm`.
    pub scope: String,
    /// `all` or `specific`.
    pub target_type: String,
    /// Panel configuration.
    pub panel: Option<Panel>,
    /// Creation time in RFC 3339 format.
    pub created_at: String,
    /// Update time in RFC 3339 format.
    pub updated_at: String,
    /// Panel version.
    pub version: i64,
}

/// A page of panel records.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelPage {
    /// Panel records.
    pub records: Vec<PanelRecord>,
    /// Cursor for the next page.
    pub next_cursor: String,
    /// Whether this is the last page.
    pub is_end: bool,
}

/// Request to create a panel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelCreate {
    /// `c2c`, `group`, `channel`, or `dm`.
    pub scope: String,
    /// `all` or `specific`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub target_type: String,
    /// Target user OpenIDs for C2C-specific panels.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub user_openids: Vec<String>,
    /// Target group OpenIDs for group-specific panels.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_openids: Vec<String>,
    /// Panel configuration.
    pub panel: Panel,
}

/// Result of creating a panel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelCreateResult {
    /// New panel ID.
    pub panel_id: String,
}

/// Request to replace a panel's items and note.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelUpdate {
    /// New panel configuration.
    pub panel: Panel,
}

/// Result of updating a panel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelUpdateResult {
    /// New panel version.
    pub version: i64,
}

/// Add or remove panel targets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelTargetUpdate {
    /// `add` or `del`.
    pub op: String,
    /// User OpenIDs for C2C panels.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub user_openids: Vec<String>,
    /// Group OpenIDs for group panels.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_openids: Vec<String>,
}

/// Detailed panel information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelDetail {
    /// Panel ID.
    pub panel_id: String,
    /// `c2c`, `group`, `channel`, or `dm`.
    pub scope: String,
    /// `all` or `specific`.
    pub target_type: String,
    /// Panel configuration.
    pub panel: Option<Panel>,
    /// Creation time.
    pub created_at: String,
    /// Update time.
    pub updated_at: String,
    /// Panel version.
    pub version: i64,
    /// Associated user OpenIDs.
    pub user_openids: Vec<String>,
    /// Associated group OpenIDs.
    pub group_openids: Vec<String>,
}
