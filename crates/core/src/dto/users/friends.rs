use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::UserId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriendView {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriendRequestUser {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriendRequestView {
    pub user: FriendRequestUser,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriendRequestsList {
    pub incoming: Vec<FriendRequestView>,
    pub outgoing: Vec<FriendRequestView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendFriendRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FriendRequestResult {
    Sent,
    Accepted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendFriendResponse {
    pub status: FriendRequestResult,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockUserRequest {
    pub user_id: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedUserView {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
}
