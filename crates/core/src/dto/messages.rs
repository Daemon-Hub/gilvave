use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{ChannelId, MessageId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ForwardedFrom {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageView {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: Option<UserId>,
    pub author_name: String,
    pub content: String,
    #[serde(default)]
    pub edited: bool,
    #[serde(default)]
    pub reply_to_id: Option<MessageId>,
    #[serde(default)]
    pub forwarded_from: Option<serde_json::Value>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default)]
    pub is_pending: bool,
}

impl MessageView {
    pub fn parse_forwarded_from(&self) -> Option<ForwardedFrom> {
        self.forwarded_from
            .as_ref()
            .and_then(|val| serde_json::from_value(val.clone()).ok())
    }
}
