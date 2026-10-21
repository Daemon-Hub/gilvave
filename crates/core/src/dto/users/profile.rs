use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::ids::UserId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ProfileView {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub settings: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserView {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub joined_at: Option<OffsetDateTime>,
    #[serde(default)]
    pub profile: ProfileView,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
}

impl UserView {
    pub fn avatar(&self) -> &str {
        if !self.profile.avatar.is_empty() {
            &self.profile.avatar
        } else if let Some(ref av) = self.avatar {
            av.as_str()
        } else {
            ""
        }
    }

    pub fn bio(&self) -> &str {
        if !self.profile.bio.is_empty() {
            &self.profile.bio
        } else if let Some(ref b) = self.bio {
            b.as_str()
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub author: String,
    pub description: String,
    pub config: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
