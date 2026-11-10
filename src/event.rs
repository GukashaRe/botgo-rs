//! Typed gateway events and payload parsing.

use serde_json::Value;

use crate::error::{Error, Result};
use crate::models::{
    AudioAction, C2cFriendData, C2cMessageStateEvent, Channel, EnterAio, EventType,
    ForumAuditResult, GroupJoinRequestEvent, GroupMemberAddEvent, GroupMemberRemoveEvent,
    GroupMessageStateEvent, GroupRobotEvent, Guild, Interaction, Member, Message, MessageAudit,
    MessageDelete, MessageReaction, Post, Reply, SubscribeMessageStatusData, Thread, WsPayload,
};

/// A typed dispatch event.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A guild was created, updated, or became available.
    GuildCreate(Guild),
    /// A guild was updated.
    GuildUpdate(Guild),
    /// A guild was removed or became unavailable.
    GuildDelete(Guild),
    /// A channel was created.
    ChannelCreate(Channel),
    /// A channel was updated.
    ChannelUpdate(Channel),
    /// A channel was deleted.
    ChannelDelete(Channel),
    /// A member joined a guild.
    GuildMemberAdd(Member),
    /// A guild member was updated.
    GuildMemberUpdate(Member),
    /// A member left a guild.
    GuildMemberRemove(Member),
    /// A guild message was created.
    MessageCreate(Message),
    /// A message mentioning the bot was created.
    AtMessageCreate(Message),
    /// A direct message was created.
    DirectMessageCreate(Message),
    /// A group message mentioning the bot was created.
    GroupAtMessageCreate(Message),
    /// A group message was created in all-message mode.
    GroupMessageCreate(Message),
    /// A C2C message was created.
    C2cMessageCreate(Message),
    /// The bot was added to a group.
    GroupAddRobot(GroupRobotEvent),
    /// The bot was removed from a group.
    GroupDelRobot(GroupRobotEvent),
    /// A user requested to join a group.
    GroupJoinRequest(GroupJoinRequestEvent),
    /// A member joined a group.
    GroupMemberAdd(GroupMemberAddEvent),
    /// A member left a group.
    GroupMemberRemove(GroupMemberRemoveEvent),
    /// Group message receiving was enabled.
    GroupMsgReceive(GroupMessageStateEvent),
    /// Group message receiving was disabled.
    GroupMsgReject(GroupMessageStateEvent),
    /// C2C message receiving was enabled.
    C2cMsgReceive(C2cMessageStateEvent),
    /// C2C message receiving was disabled.
    C2cMsgReject(C2cMessageStateEvent),
    /// A guild message was deleted.
    MessageDelete(MessageDelete),
    /// A public message was deleted.
    PublicMessageDelete(MessageDelete),
    /// A direct message was deleted.
    DirectMessageDelete(MessageDelete),
    /// A reaction was added.
    MessageReactionAdd(MessageReaction),
    /// A reaction was removed.
    MessageReactionRemove(MessageReaction),
    /// An audio stream started.
    AudioStart(AudioAction),
    /// An audio stream finished.
    AudioFinish(AudioAction),
    /// The bot moved onto the microphone.
    AudioOnMic(AudioAction),
    /// The bot moved off the microphone.
    AudioOffMic(AudioAction),
    /// A message passed audit.
    MessageAuditPass(MessageAudit),
    /// A message failed audit.
    MessageAuditReject(MessageAudit),
    /// A forum thread was created.
    ForumThreadCreate(Thread),
    /// A forum thread was updated.
    ForumThreadUpdate(Thread),
    /// A forum thread was deleted.
    ForumThreadDelete(Thread),
    /// A forum post was created.
    ForumPostCreate(Post),
    /// A forum post was deleted.
    ForumPostDelete(Post),
    /// A forum reply was created.
    ForumReplyCreate(Reply),
    /// A forum reply was deleted.
    ForumReplyDelete(Reply),
    /// A forum publish audit finished.
    ForumAudit(ForumAuditResult),
    /// An interaction was created.
    InteractionCreate(Interaction),
    /// Subscription authorization status changed.
    SubscribeMessageStatus(SubscribeMessageStatusData),
    /// A C2C friend was added.
    FriendAdd(C2cFriendData),
    /// A C2C friend was removed.
    FriendDelete(C2cFriendData),
    /// A user entered an AIO session.
    EnterAio(EnterAio),
    /// An event type unknown to this version of the SDK.
    Unknown {
        /// Event type reported by the gateway.
        event_type: EventType,
        /// Raw event data.
        data: Value,
    },
}

impl Event {
    /// Returns the wire event type.
    #[must_use]
    pub fn event_type(&self) -> EventType {
        match self {
            Self::GuildCreate(_) => EventType::GUILD_CREATE,
            Self::GuildUpdate(_) => EventType::GUILD_UPDATE,
            Self::GuildDelete(_) => EventType::GUILD_DELETE,
            Self::ChannelCreate(_) => EventType::CHANNEL_CREATE,
            Self::ChannelUpdate(_) => EventType::CHANNEL_UPDATE,
            Self::ChannelDelete(_) => EventType::CHANNEL_DELETE,
            Self::GuildMemberAdd(_) => EventType::GUILD_MEMBER_ADD,
            Self::GuildMemberUpdate(_) => EventType::GUILD_MEMBER_UPDATE,
            Self::GuildMemberRemove(_) => EventType::GUILD_MEMBER_REMOVE,
            Self::MessageCreate(_) => EventType::MESSAGE_CREATE,
            Self::AtMessageCreate(_) => EventType::AT_MESSAGE_CREATE,
            Self::DirectMessageCreate(_) => EventType::DIRECT_MESSAGE_CREATE,
            Self::GroupAtMessageCreate(_) => EventType::GROUP_AT_MESSAGE_CREATE,
            Self::GroupMessageCreate(_) => EventType::GROUP_MESSAGE_CREATE,
            Self::C2cMessageCreate(_) => EventType::C2C_MESSAGE_CREATE,
            Self::GroupAddRobot(_) => EventType::GROUP_ADD_ROBOT,
            Self::GroupDelRobot(_) => EventType::GROUP_DEL_ROBOT,
            Self::GroupJoinRequest(_) => EventType::GROUP_JOIN_REQUEST,
            Self::GroupMemberAdd(_) => EventType::GROUP_MEMBER_ADD,
            Self::GroupMemberRemove(_) => EventType::GROUP_MEMBER_REMOVE,
            Self::GroupMsgReceive(_) => EventType::GROUP_MSG_RECEIVE,
            Self::GroupMsgReject(_) => EventType::GROUP_MSG_REJECT,
            Self::C2cMsgReceive(_) => EventType::C2C_MSG_RECEIVE,
            Self::C2cMsgReject(_) => EventType::C2C_MSG_REJECT,
            Self::MessageDelete(_) => EventType::MESSAGE_DELETE,
            Self::PublicMessageDelete(_) => EventType::PUBLIC_MESSAGE_DELETE,
            Self::DirectMessageDelete(_) => EventType::DIRECT_MESSAGE_DELETE,
            Self::MessageReactionAdd(_) => EventType::MESSAGE_REACTION_ADD,
            Self::MessageReactionRemove(_) => EventType::MESSAGE_REACTION_REMOVE,
            Self::AudioStart(_) => EventType::AUDIO_START,
            Self::AudioFinish(_) => EventType::AUDIO_FINISH,
            Self::AudioOnMic(_) => EventType::AUDIO_ON_MIC,
            Self::AudioOffMic(_) => EventType::AUDIO_OFF_MIC,
            Self::MessageAuditPass(_) => EventType::MESSAGE_AUDIT_PASS,
            Self::MessageAuditReject(_) => EventType::MESSAGE_AUDIT_REJECT,
            Self::ForumThreadCreate(_) => EventType::FORUM_THREAD_CREATE,
            Self::ForumThreadUpdate(_) => EventType::FORUM_THREAD_UPDATE,
            Self::ForumThreadDelete(_) => EventType::FORUM_THREAD_DELETE,
            Self::ForumPostCreate(_) => EventType::FORUM_POST_CREATE,
            Self::ForumPostDelete(_) => EventType::FORUM_POST_DELETE,
            Self::ForumReplyCreate(_) => EventType::FORUM_REPLY_CREATE,
            Self::ForumReplyDelete(_) => EventType::FORUM_REPLY_DELETE,
            Self::ForumAudit(_) => EventType::FORUM_PUBLISH_AUDIT_RESULT,
            Self::InteractionCreate(_) => EventType::INTERACTION_CREATE,
            Self::SubscribeMessageStatus(_) => EventType::SUBSCRIBE_MESSAGE_STATUS,
            Self::FriendAdd(_) => EventType::FRIEND_ADD,
            Self::FriendDelete(_) => EventType::FRIEND_DEL,
            Self::EnterAio(_) => EventType::ENTER_AIO,
            Self::Unknown { event_type, .. } => event_type.clone(),
        }
    }

    /// Parses a dispatch payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not a dispatch event, has no event
    /// type, or contains data that does not match the event schema.
    pub fn from_payload(payload: &WsPayload) -> Result<Self> {
        if payload.op != crate::models::OpCode::DISPATCH {
            return Err(Error::Protocol(format!(
                "expected a dispatch opcode, got {}",
                payload.op
            )));
        }
        let event_type = payload
            .event_type
            .clone()
            .ok_or_else(|| Error::Protocol("dispatch payload has no event type".to_owned()))?;
        let data = payload.data.clone().unwrap_or(Value::Null);

        macro_rules! parse {
            ($variant:ident) => {
                Ok(Self::$variant(
                    serde_json::from_value(data).map_err(Error::from)?,
                ))
            };
        }

        match event_type.as_str() {
            "GUILD_CREATE" => parse!(GuildCreate),
            "GUILD_UPDATE" => parse!(GuildUpdate),
            "GUILD_DELETE" => parse!(GuildDelete),
            "CHANNEL_CREATE" => parse!(ChannelCreate),
            "CHANNEL_UPDATE" => parse!(ChannelUpdate),
            "CHANNEL_DELETE" => parse!(ChannelDelete),
            "GUILD_MEMBER_ADD" => parse!(GuildMemberAdd),
            "GUILD_MEMBER_UPDATE" => parse!(GuildMemberUpdate),
            "GUILD_MEMBER_REMOVE" => parse!(GuildMemberRemove),
            "MESSAGE_CREATE" => parse!(MessageCreate),
            "AT_MESSAGE_CREATE" => parse!(AtMessageCreate),
            "DIRECT_MESSAGE_CREATE" => parse!(DirectMessageCreate),
            "GROUP_AT_MESSAGE_CREATE" => parse!(GroupAtMessageCreate),
            "GROUP_MESSAGE_CREATE" => parse!(GroupMessageCreate),
            "C2C_MESSAGE_CREATE" => parse!(C2cMessageCreate),
            "GROUP_ADD_ROBOT" => parse!(GroupAddRobot),
            "GROUP_DEL_ROBOT" => parse!(GroupDelRobot),
            "GROUP_JOIN_REQUEST" => parse!(GroupJoinRequest),
            "GROUP_MEMBER_ADD" => parse!(GroupMemberAdd),
            "GROUP_MEMBER_REMOVE" => parse!(GroupMemberRemove),
            "GROUP_MSG_RECEIVE" => parse!(GroupMsgReceive),
            "GROUP_MSG_REJECT" => parse!(GroupMsgReject),
            "C2C_MSG_RECEIVE" => parse!(C2cMsgReceive),
            "C2C_MSG_REJECT" => parse!(C2cMsgReject),
            "MESSAGE_DELETE" => parse!(MessageDelete),
            "PUBLIC_MESSAGE_DELETE" => parse!(PublicMessageDelete),
            "DIRECT_MESSAGE_DELETE" => parse!(DirectMessageDelete),
            "MESSAGE_REACTION_ADD" => parse!(MessageReactionAdd),
            "MESSAGE_REACTION_REMOVE" => parse!(MessageReactionRemove),
            "AUDIO_START" => parse!(AudioStart),
            "AUDIO_FINISH" => parse!(AudioFinish),
            "AUDIO_ON_MIC" => parse!(AudioOnMic),
            "AUDIO_OFF_MIC" => parse!(AudioOffMic),
            "MESSAGE_AUDIT_PASS" => parse!(MessageAuditPass),
            "MESSAGE_AUDIT_REJECT" => parse!(MessageAuditReject),
            "FORUM_THREAD_CREATE" => parse!(ForumThreadCreate),
            "FORUM_THREAD_UPDATE" => parse!(ForumThreadUpdate),
            "FORUM_THREAD_DELETE" => parse!(ForumThreadDelete),
            "FORUM_POST_CREATE" => parse!(ForumPostCreate),
            "FORUM_POST_DELETE" => parse!(ForumPostDelete),
            "FORUM_REPLY_CREATE" => parse!(ForumReplyCreate),
            "FORUM_REPLY_DELETE" => parse!(ForumReplyDelete),
            "FORUM_PUBLISH_AUDIT_RESULT" => parse!(ForumAudit),
            "INTERACTION_CREATE" => parse!(InteractionCreate),
            "SUBSCRIBE_MESSAGE_STATUS" => parse!(SubscribeMessageStatus),
            "FRIEND_ADD" => parse!(FriendAdd),
            "FRIEND_DEL" => parse!(FriendDelete),
            "ENTER_AIO" => parse!(EnterAio),
            _ => Ok(Self::Unknown { event_type, data }),
        }
    }
}

/// Extracts the `d` field from a raw gateway or webhook body.
///
/// # Errors
///
/// Returns an error when the body is not valid JSON or has no `d` field.
pub fn parse_data<T>(body: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let value: Value = serde_json::from_slice(body)?;
    let data = value
        .get("d")
        .ok_or_else(|| Error::Protocol("payload has no data field".to_owned()))?;
    serde_json::from_value(data.clone()).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_message_event() {
        let payload: WsPayload = serde_json::from_str(
            r#"{
                "op": 0,
                "s": 1,
                "t": "AT_MESSAGE_CREATE",
                "d": {
                    "id": "m1",
                    "channel_id": "c1",
                    "content": "<@!123> hello",
                    "author": {"id": "u1", "username": "tester"}
                }
            }"#,
        )
        .unwrap();

        let event = Event::from_payload(&payload).unwrap();
        assert_eq!(event.event_type(), EventType::AT_MESSAGE_CREATE);
        match event {
            Event::AtMessageCreate(message) => {
                assert_eq!(message.content, "<@!123> hello");
                assert_eq!(message.author.unwrap().id, "u1");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn keeps_unknown_events() {
        let payload: WsPayload =
            serde_json::from_str(r#"{"op":0,"t":"SOMETHING_NEW","d":{"x":1}}"#).unwrap();
        let event = Event::from_payload(&payload).unwrap();
        assert_eq!(event.event_type().as_str(), "SOMETHING_NEW");
    }
}
