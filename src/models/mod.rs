//! Data transfer objects for the QQ Bot OpenAPI and gateway.
//!
//! The module layout follows the Go SDK's `dto` package, while names use
//! idiomatic Rust casing and enums are represented by their integer wire
//! values with named constants in submodules.

pub mod common;
pub mod gateway;
pub mod group;
pub mod interaction;
pub mod keyboard;
pub mod menu;
pub mod message;
pub mod misc;
pub mod panel;
pub mod upload;

pub use common::{
    Channel, ChannelValueObject, DEFAULT_COLOR, DELETE_ALL, DELETE_FIFTEEN_DAYS, DELETE_SEVEN_DAYS,
    DELETE_THIRTY_DAYS, DELETE_THREE_DAYS, DeleteHistoryMsgDay, DeleteHistoryMsgDays, Duration,
    Guild, GuildMembersPager, GuildPager, GuildRoleMembersPager, GuildRoles, Member,
    MemberAddRoleBody, MemberDeleteOptions, MessagePagerType, MessageReactionPager, MessagesPager,
    NO_DELETE, Role, RoleId, Timestamp, UpdateResult, UpdateRole, UpdateRoleFilter, UpdateRoleInfo,
    User, channel_type,
};
pub use gateway::{
    EventType, GatewayUrl, Intents, OpCode, SessionStartLimit, ShardConfig, WebsocketAp,
    WsHelloData, WsIdentityData, WsPayload, WsProperties, WsReadyData, WsResumeData, WsUser,
};
pub use group::{
    AutoApproved, BatchRemoveMembers, BatchRemoveMembersResult, C2cMessageStateEvent,
    CreateJoinApprovalStrategy, GlobalMuteRule, GroupBlacklistPage, GroupBlacklistResult,
    GroupBlacklistUpdate, GroupBlacklistUser, GroupBotState, GroupInfo, GroupJoinRequest,
    GroupJoinRequestApproval, GroupJoinRequestEvent, GroupJoinRequestPage, GroupMember,
    GroupMemberAddEvent, GroupMemberList, GroupMemberRemoveEvent, GroupMessageStateEvent,
    GroupRestrictChatSetting, GroupRobotEvent, JoinApprovalGroupAction, JoinApprovalStrategy,
    JoinApprovalStrategyPage, JoinApprovalStrategyResult, JoinApprovalWhitelistResult,
    JoinApprovalWhitelistUpdate, MemberMuteState, MuteRecurringRule, MuteScheduleRule, ReviewQa,
    SetGroupMemberMutes, SetMemberMuteState, UpdateJoinApprovalStrategy, VerifyInfo,
};
pub use interaction::{
    AuthorizeData, C2cFriendData, Emoji, EnterAio, ForumAuditResult, Interaction, InteractionData,
    InteractionMessageScene, MessageReaction, Post, PostInfo, ReactionTarget, Reply, ReplyInfo,
    Resolved, SearchInputResolved, SearchLayout, SearchRecord, SearchResponse,
    SubscribeMessageStatusData, SubscribeMsgTemplateResult, Thread, ThreadInfo,
    interaction_data_type, interaction_type,
};
pub use keyboard::{
    Action, Button, CustomKeyboard, KeyboardStyle, MessageKeyboard, Modal, Permission, RenderData,
    Row, SubscribeData, TemplateId, action_type, permission_type,
};
pub use menu::{
    Menu, MenuItem, MenuResponse, MenuSwitch, MenuUpdate, MenuUpdateResult, SubMenuItem,
};
pub use message::{
    ActionButton, ApiMessage, Ark, ArkData, ArkKv, ArkObj, ArkObjKv, Embed, EmbedField,
    InputNotify, Markdown, MarkdownParams, MarkdownStyle, MediaInfo, Message, MessageArk,
    MessageAttachment, MessageAudit, MessageCard, MessageCardContent, MessageDelete,
    MessageEmbedThumbnail, MessageReactionUsers, MessageReference, MessageScene, MessageSetting,
    MessageToCreate, MsgElement, PromptKeyboard, RichMediaMessage, SendType, SettingGuide,
    SettingGuideToCreate, Stream, message_type,
};
pub use misc::{
    Announces, ApiPermission, ApiPermissionDemand, ApiPermissionDemandIdentify,
    ApiPermissionDemandToCreate, ApiPermissions, AudioAction, AudioControl,
    ChannelAnnouncesToCreate, ChannelPermissions, ChannelRolesPermissions, DirectMessage,
    DirectMessageToCreate, GuildAnnouncesToCreate, HttpBot, HttpIdentity, HttpReady, HttpSession,
    PinsMessage, RecommendChannel, Schedule, ScheduleWrapper, UpdateChannelPermissions,
    UpdateGuildMute, UpdateGuildMuteResponse, WebhookAck, WebhookValidationRequest,
    WebhookValidationResponse, audio_status,
};
pub use panel::{
    Panel, PanelCreate, PanelCreateResult, PanelDetail, PanelItem, PanelPage, PanelRecord,
    PanelTargetUpdate, PanelUpdate, PanelUpdateResult,
};
pub use upload::{
    FileUploadRequest, FileUploadResponse, MediaImage, MediaImageCompleteRequest,
    MediaImageCompleteResponse, MediaImagePage, MediaImagePrepareRequest,
    MediaImagePrepareResponse, MessageExtInfo, StreamMessageRequest, StreamMessageResponse,
    UploadConfig, UploadPart, UploadPartFinish, UploadPrepareRequest, UploadPrepareResponse,
    UrlLinkData, UrlLinkRequest, UrlLinkResponse,
};
