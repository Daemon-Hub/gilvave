use serde::{Deserialize, Serialize};

use crate::ids::UserId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_info: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTokensRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserView {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub avatar: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SessionView {
    pub id: String,
    pub device_info: crate::settings::DeviceInfo,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub last_used_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(default)]
    pub is_current: bool,
}

impl SessionView {
    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.device_info.device_name
            && !name.trim().is_empty()
        {
            if let Some(ref plat) = self.device_info.platform {
                return format!("{name} ({plat})");
            }
            return name.clone();
        }

        let plat = self
            .device_info
            .platform
            .as_deref()
            .unwrap_or("Неизвестное устройство");

        match self.device_info.client.as_str() {
            "desktop" => format!("{plat} • Gilvave Desktop"),
            "web" => {
                if let Some(ref b) = self.device_info.browser {
                    format!("{b} • {plat}")
                } else {
                    format!("{plat} • Web Browser")
                }
            }
            "mobile" => format!("{plat} • Мобильное приложение"),
            _ => plat.to_string(),
        }
    }

    pub fn client_kind(&self) -> &'static str {
        match self.device_info.client.as_str() {
            "desktop" => "desktop",
            "mobile" => "mobile",
            _ => "web",
        }
    }
}
