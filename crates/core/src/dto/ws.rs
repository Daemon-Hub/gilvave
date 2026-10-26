use serde::{Deserialize, Serialize};

use crate::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId},
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
    MessageNew(MessageView),
    MessageUpdate(MessageView),
    MessageDelete {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    ChannelHistoryBefore(Vec<MessageView>),
    ChannelHistoryAfter(Vec<MessageView>),
    // TODO [WS]: VoiceStateUpdate { channel_id: ChannelId, user_id: UserId, is_muted: bool, is_deafened: bool },
    // TODO [WS]: VoiceParticipantJoined { channel_id: ChannelId, user_id: UserId, username: String, avatar: String },
    // TODO [WS]: VoiceParticipantLeft { channel_id: ChannelId, user_id: UserId },
    // TODO [WS]: VoiceSpeaking { channel_id: ChannelId, user_id: UserId, is_speaking: bool },
    // TODO [WS]: VoiceSignal { channel_id: ChannelId, sender_id: UserId, data: String } (WebRTC SDP/ICE signaling),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerSend {
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
    // TODO [WS]: JoinVoiceChannel { channel_id: ChannelId },
    // TODO [WS]: LeftVoiceChannel { channel_id: ChannelId },
    // TODO [WS]: VoiceStateUpdate { channel_id: ChannelId, is_muted: bool, is_deafened: bool },
    // TODO [WS]: VoiceSignal { channel_id: ChannelId, target_id: Option<UserId>, data: String } (WebRTC SDP/ICE signaling),
}
