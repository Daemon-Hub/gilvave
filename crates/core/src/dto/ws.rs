use serde::{Deserialize, Serialize};

use crate::{
    dto::{
        messages::MessageView,
        voice::{VoiceParticipantView, VoiceSignalData},
    },
    ids::{ChannelId, MessageId, ServerId, UserId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerRecieve {
    HeartbeatAck,
    Hello,
    Error {
        message: String,
    },
    JoinSuccess,
    #[serde(alias = "MESSAGE_CREATE")]
    MessageNew(MessageView),
    MessageUpdate(MessageView),
    MessageDelete {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    ChannelHistoryBefore(Vec<MessageView>),
    ChannelHistoryAfter(Vec<MessageView>),
    ServerOnline {
        server_id: ServerId,
        online_users: Vec<UserId>,
    },
    UserOnline {
        user_id: UserId,
    },
    UserOffline {
        user_id: UserId,
    },
    VoiceRoomJoined {
        channel_id: ChannelId,
        participants: Vec<VoiceParticipantView>,
    },
    VoiceParticipantJoined {
        channel_id: ChannelId,
        participant: VoiceParticipantView,
    },
    VoiceParticipantLeft {
        channel_id: ChannelId,
        user_id: UserId,
    },
    VoiceStateUpdate {
        channel_id: ChannelId,
        user_id: UserId,
        is_muted: bool,
        is_deafened: bool,
    },
    VoiceSignal {
        channel_id: ChannelId,
        signal: VoiceSignalData,
    },
    VoiceSpeaking {
        channel_id: ChannelId,
        user_id: UserId,
        is_speaking: bool,
    },
    VoiceVideoState {
        channel_id: ChannelId,
        user_id: UserId,
        is_camera_on: bool,
        is_screen_sharing: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerSend {
    Heartbeat,
    MessageCreate {
        channel_id: ChannelId,
        content: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reply_to_id: Option<MessageId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        forwarded_from: Option<serde_json::Value>,
    },
    #[serde(alias = "MESSAGE_EDIT")]
    MessageUpdate {
        channel_id: ChannelId,
        message_id: MessageId,
        content: String,
    },
    MessageDelete {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    ServerOnline {
        server_id: ServerId,
    },
    JoinChannel {
        channel_id: ChannelId,
    },
    LeftChannel {
        channel_id: ChannelId,
    },
    ChannelHistoryBefore {
        channel_id: ChannelId,
        #[serde(with = "time::serde::rfc3339")]
        timestamp: time::OffsetDateTime,
    },
    ChannelHistoryAfter {
        channel_id: ChannelId,
        #[serde(with = "time::serde::rfc3339")]
        timestamp: time::OffsetDateTime,
    },
    JoinVoiceChannel {
        channel_id: ChannelId,
    },
    LeftVoiceChannel {
        channel_id: ChannelId,
    },
    VoiceStateUpdate {
        channel_id: ChannelId,
        is_muted: bool,
        is_deafened: bool,
    },
    VoiceSignal {
        channel_id: ChannelId,
        signal: VoiceSignalData,
    },
    VoiceSpeaking {
        channel_id: ChannelId,
        is_speaking: bool,
    },
    VoiceVideoState {
        channel_id: ChannelId,
        is_camera_on: bool,
        is_screen_sharing: bool,
    },
}
