use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    dto::{
        messages::MessageView,
        servers::{ChannelView, ServerSmallPart},
    },
    error::ErrorInfo,
    ids::{ChannelId, MessageId, ServerId},
    settings::DeviceInfo,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandArgs {
    GetAccessToken,
    GetRefreshToken,
    SetAccessToken {
        token: String,
    },
    SetRefreshToken {
        token: String,
    },
    GetDeviceInfo,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowStartDragging,
    OpenDevTools,
    // Offline Cache Commands (Desktop/Mobile SQLite cache)
    GetCachedMessages {
        channel_id: ChannelId,
        limit: usize,
        before: Option<MessageId>,
    },
    SyncCachedMessages {
        channel_id: ChannelId,
        messages: Vec<MessageView>,
    },
    SaveCachedMessage {
        message: MessageView,
    },
    UpdateCachedMessage {
        id: MessageId,
        content: String,
        edited: bool,
    },
    DeleteCachedMessage {
        id: MessageId,
    },
    GetCachedServers,
    SyncCachedServers {
        servers: Vec<ServerSmallPart>,
    },
    GetCachedChannels {
        server_id: ServerId,
    },
    SyncCachedChannels {
        server_id: ServerId,
        channels: Vec<ChannelView>,
    },
}

impl CommandArgs {
    pub fn to_json(self) -> Value {
        json!({"command": self})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandResponse {
    GetAccessToken(String),
    GetRefreshToken(String),
    SetAccessToken,
    SetRefreshToken,
    GetDeviceInfo(DeviceInfo),
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowStartDragging,
    OpenDevTools,
    // Offline Cache Responses
    GetCachedMessages(Vec<MessageView>),
    SyncCachedMessages,
    SaveCachedMessage,
    UpdateCachedMessage,
    DeleteCachedMessage,
    GetCachedServers(Vec<ServerSmallPart>),
    SyncCachedServers,
    GetCachedChannels(Vec<ChannelView>),
    SyncCachedChannels,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandResult {
    Ok(CommandResponse),
    Error(ErrorInfo),
}

impl CommandResult {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok(_) => true,
            Self::Error(_) => false,
        }
    }
}
