use serde::{Deserialize, Serialize};

use crate::ids::ChannelId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    TEXT,
    VOICE,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelView {
    pub id: ChannelId,
    pub name: String,
    pub r#type: ChannelType,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChannelCreateInfo {
    pub name: String,
    pub r#type: ChannelType,
}
