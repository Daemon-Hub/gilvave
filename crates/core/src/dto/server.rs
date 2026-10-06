use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{ServerId, UserId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServerSmallPart {
    pub id: ServerId,
    pub name: String,
    pub icon_url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Server {
    pub id: ServerId,
    pub owner_id: UserId,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub cover: String,
    pub is_public: bool,
    pub members_count: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct MemberView {
    pub user_id: UserId,
    pub username: String,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServerCreateInfo {
    pub name: String,
    pub is_public: bool,
}
