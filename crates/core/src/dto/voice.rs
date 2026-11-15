use serde::{Deserialize, Serialize};

use crate::ids::{ChannelId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoiceSessionType {
    Publisher,
    Subscriber,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoiceSignalData {
    Offer {
        session: VoiceSessionType,
        sdp: String,
    },
    Answer {
        session: VoiceSessionType,
        sdp: String,
    },
    IceCandidate {
        session: VoiceSessionType,
        candidate: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sdp_mid: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sdp_mline_index: Option<u16>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VideoCodecPreference {
    #[default]
    Auto,
    Vp8,
    H264,
    Vp9,
    Av1,
}

impl VideoCodecPreference {
    pub fn mime_type(&self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::Vp8 => Some("video/VP8"),
            Self::H264 => Some("video/H264"),
            Self::Vp9 => Some("video/VP9"),
            Self::Av1 => Some("video/AV1"),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Авто (VP8)",
            Self::Vp8 => "VP8 (Базовый)",
            Self::H264 => "H.264 (Аппаратный)",
            Self::Vp9 => "VP9 (Высокая четкость)",
            Self::Av1 => "AV1 (Максимальное сжатие)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Auto => "Авто",
            Self::Vp8 => "VP8",
            Self::H264 => "H.264",
            Self::Vp9 => "VP9",
            Self::Av1 => "AV1",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceParticipantView {
    pub user_id: UserId,
    pub username: String,
    pub avatar: String,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_speaking: bool,
    #[serde(default)]
    pub is_camera_on: bool,
    #[serde(default)]
    pub is_screen_sharing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceRoomStateView {
    pub channel_id: ChannelId,
    pub participants: Vec<VoiceParticipantView>,
}
