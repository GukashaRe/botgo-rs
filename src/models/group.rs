//! QQ group management API and event models.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

use super::common::Timestamp;

/// Basic information about a QQ group.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupInfo {
    /// Group OpenID.
    pub group_openid: String,
    /// Group name.
    pub group_name: String,
    /// Group introduction.
    pub group_finger_memo: String,
    /// Group category.
    pub group_class_text: String,
    /// Group tags.
    pub group_tags: Vec<String>,
    /// Number of group members.
    pub group_member_num: i32,
}

/// The bot's state inside a group.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupBotState {
    /// Bot member OpenID.
    pub member_openid: String,
    /// Join time in RFC 3339 format.
    pub joined_at: Timestamp,
    /// Whether proactive messages are accepted.
    pub allow_proactive_msg: bool,
    /// Message receiving mode: `all`, `only_mention`, or `mention_and_context`.
    pub recv_msg_setting: String,
    /// Bot role: `member`, `owner`, or `admin`.
    pub member_role: String,
}

/// A member of a QQ group.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupMember {
    /// Member OpenID.
    pub member_openid: String,
    /// Display name.
    pub username: String,
    /// Group role: `member`, `owner`, or `admin`.
    pub member_role: String,
    /// Whether this member is a bot.
    pub bot: bool,
    /// Join time in RFC 3339 format.
    pub joined_at: Timestamp,
    /// Union OpenID, when available.
    pub union_openid: String,
}

/// A page of group members.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupMemberList {
    /// Members, at most 30 per page.
    pub members: Vec<GroupMember>,
    /// Cursor for the next page; empty at the end.
    pub next_cursor: String,
}

/// A user on a group blacklist.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupBlacklistUser {
    /// Union OpenID, when available.
    pub union_openid: String,
    /// Member OpenID.
    pub member_openid: String,
    /// Display name.
    pub username: String,
    /// Blacklist time in RFC 3339 format.
    pub banned_at: Timestamp,
    /// Whether this user is a bot.
    pub bot: bool,
}

/// A page of group blacklist users.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupBlacklistPage {
    /// Blacklisted users.
    pub users: Vec<GroupBlacklistUser>,
    /// Cursor for the next page; empty at the end.
    pub next_cursor: String,
}

/// Add or remove group blacklist entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupBlacklistUpdate {
    /// `add` or `del`.
    pub op: String,
    /// Target member OpenIDs, at most 20.
    pub member_openids: Vec<String>,
}

/// Result of a group blacklist operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupBlacklistResult {
    /// OpenIDs that could not be updated.
    pub fail_openids: Vec<String>,
}

/// Request to remove multiple group members.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BatchRemoveMembers {
    /// Member OpenIDs to remove, at most 20.
    pub member_openids: Vec<String>,
    /// Also add removed members to the blacklist.
    #[serde(skip_serializing_if = "is_false")]
    pub add_to_member_blacklist: bool,
}

const fn is_false(value: &bool) -> bool {
    !*value
}

/// Result of a bulk member removal.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BatchRemoveMembersResult {
    /// `success` when the members were removed.
    pub remove_members_result: String,
    /// OpenIDs that could not be added to the blacklist.
    pub add_to_member_blacklist_fail_openids: Vec<String>,
}

/// One answer in an administrator review Q&A.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReviewQa {
    /// Question configured by the administrator.
    pub question: String,
    /// Answer supplied by the applicant.
    pub answer: String,
}

/// Verification information attached to a join request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VerifyInfo {
    /// `verify_message` or `admin_review_qa`.
    pub method: String,
    /// Verification message.
    pub verify_message: String,
    /// Administrator review questions.
    pub review_qa_list: Vec<ReviewQa>,
}

/// Information about an automatically approved join request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoApproved {
    /// Strategy ID that approved the request.
    pub strategy_id: String,
}

/// A pending group join request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupJoinRequest {
    /// Request ID, required when approving.
    pub join_request_id: String,
    /// Risk warning text.
    pub risk_tips: String,
    /// Union OpenID, when available.
    pub union_openid: String,
    /// Applicant OpenID.
    pub member_openid: String,
    /// Applicant nickname.
    pub username: String,
    /// Application time in RFC 3339 format.
    pub apply_at: Timestamp,
    /// `self_apply` or `invited`.
    pub apply_source: String,
    /// Inviter OpenID when `apply_source` is `invited`.
    pub invited_by: String,
    /// Whether the applicant is a bot.
    pub bot: bool,
    /// Verification details.
    pub verify_info: Option<VerifyInfo>,
    /// Automatic approval details, present on downstream events.
    pub auto_approved: Option<AutoApproved>,
}

/// A page of group join requests.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupJoinRequestPage {
    /// Join requests.
    pub list: Vec<GroupJoinRequest>,
    /// Cursor for the next page; empty at the end.
    pub next_cursor: String,
}

/// Approve or reject a group join request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupJoinRequestApproval {
    /// `approve` or `decline`.
    pub op: String,
    /// Join request ID from the event or list.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub join_request_id: String,
    /// Rejection reason, used with `decline`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reject_reason: String,
    /// Also add the applicant to the blacklist when rejecting.
    #[serde(skip_serializing_if = "is_false")]
    pub add_to_member_blacklist: bool,
}

/// A scheduled mute rule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MuteScheduleRule {
    /// Task ID.
    pub task_id: String,
    /// Start time in RFC 3339 format.
    pub start_at: Timestamp,
    /// End time in RFC 3339 format.
    pub end_at: Timestamp,
    /// Whether the rule is enabled.
    pub enabled: bool,
}

/// A recurring weekly mute rule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MuteRecurringRule {
    /// Task ID.
    pub task_id: String,
    /// Weekdays, 1 = Monday through 7 = Sunday.
    pub weekdays: Vec<i32>,
    /// Start time in `HH:mm` format.
    pub start_time: String,
    /// End time in `HH:mm` format. May cross midnight.
    pub end_time: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
}

/// Guild-wide mute configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalMuteRule {
    /// `none`, `always`, or `schedule`.
    pub mode: String,
    /// Scheduled mute rules.
    pub schedule_rules: Vec<MuteScheduleRule>,
    /// Recurring mute rules.
    pub recurring_rules: Vec<MuteRecurringRule>,
}

/// A currently muted group member.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MemberMuteState {
    /// Member OpenID.
    pub member_openid: String,
    /// Mute expiry in RFC 3339 format.
    pub mute_expire_at: Timestamp,
    /// Display name.
    pub username: String,
    /// Union OpenID, when available.
    pub union_openid: String,
}

/// Current group mute state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupRestrictChatSetting {
    /// Group-wide mute rules.
    pub global_rule: GlobalMuteRule,
    /// Currently muted members.
    pub members: Vec<MemberMuteState>,
}

/// A member-level mute update.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SetMemberMuteState {
    /// `add`, `update`, or `del`.
    pub op: String,
    /// Target member OpenID.
    pub member_openid: String,
    /// Mute expiry in RFC 3339 format; may be empty when `op` is `del`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mute_expire_at: String,
}

/// Request to update member-level mutes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SetGroupMemberMutes {
    /// Mute updates, at most 20.
    pub members: Vec<SetMemberMuteState>,
}

/// A join approval strategy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct JoinApprovalStrategy {
    /// Strategy ID.
    pub strategy_id: String,
    /// Associated group OpenIDs.
    pub group_openids: Vec<String>,
    /// Associated QQ group numbers, returned as strings by the server.
    pub group_ids: Vec<String>,
    /// Estimated number of whitelisted QQ numbers.
    pub whitelist_user_count: i32,
    /// `on` or `off`.
    pub is_enable: String,
    /// Expiry time in RFC 3339 format.
    pub expire_at: Timestamp,
    /// Creation time in RFC 3339 format.
    pub created_at: Timestamp,
    /// Last update time in RFC 3339 format.
    pub updated_at: Timestamp,
    /// Developer note.
    pub remark: String,
}

/// A page of join approval strategies.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct JoinApprovalStrategyPage {
    /// Active strategies.
    pub strategies: Vec<JoinApprovalStrategy>,
    /// Cursor for the next page; empty at the end.
    pub next_cursor: String,
}

/// Request to create a join approval strategy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CreateJoinApprovalStrategy {
    /// Associated group OpenIDs, mutually exclusive with `group_ids`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_openids: Vec<String>,
    /// Associated QQ group numbers, mutually exclusive with `group_openids`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_ids: Vec<u64>,
    /// `on` or `off`; defaults to `on`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub is_enable: String,
    /// Expiry time in RFC 3339 format.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub expire_at: String,
    /// Developer note.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remark: String,
}

/// Add or remove associated groups from a strategy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct JoinApprovalGroupAction {
    /// `add` or `del`.
    pub op: String,
    /// Group OpenIDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_openids: Vec<String>,
    /// QQ group numbers.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_ids: Vec<u64>,
}

/// Request to update a join approval strategy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateJoinApprovalStrategy {
    /// `on` or `off`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub is_enable: String,
    /// Expiry time in RFC 3339 format.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub expire_at: String,
    /// Associated group additions or removals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_action: Option<JoinApprovalGroupAction>,
    /// Developer note.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remark: String,
}

/// Result of creating or updating a strategy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct JoinApprovalStrategyResult {
    /// Strategy ID.
    pub strategy_id: String,
    /// `on` or `off`.
    pub is_enable: String,
    /// Expiry time in RFC 3339 format.
    pub expire_at: Timestamp,
}

/// Update the whitelist of a strategy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinApprovalWhitelistUpdate {
    /// `add` or `del`.
    pub op: String,
    /// QQ numbers as strings.
    pub whitelist_users: Vec<String>,
}

/// Result of a whitelist update.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct JoinApprovalWhitelistResult {
    /// Strategy ID.
    pub strategy_id: String,
    /// Estimated number of whitelisted numbers.
    pub whitelist_user_count: i32,
    /// Update time in RFC 3339 format.
    pub updated_at: Timestamp,
}

/// A bot added to or removed from a group.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupRobotEvent {
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    /// Group OpenID.
    pub group_openid: String,
    /// Member OpenID that performed the operation.
    pub op_member_openid: String,
}

/// A group message receive setting change.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupMessageStateEvent {
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    /// Group OpenID.
    pub group_openid: String,
    /// Member OpenID that performed the operation.
    pub op_member_openid: String,
}

/// A member joined a group.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupMemberAddEvent {
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    /// Group OpenID.
    pub group_openid: String,
    /// Joined member OpenID.
    pub member_openid: String,
}

/// A member left a group.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupMemberRemoveEvent {
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    /// Group OpenID.
    pub group_openid: String,
    /// Member OpenID.
    pub member_openid: String,
    /// User OpenID, when available.
    pub user_openid: String,
}

/// A C2C message receive setting change.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct C2cMessageStateEvent {
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    /// User OpenID.
    pub openid: String,
}

/// A group join request event, including the group OpenID.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupJoinRequestEvent {
    /// Group OpenID.
    pub group_openid: String,
    /// Join request details.
    #[serde(flatten)]
    pub request: GroupJoinRequest,
}
