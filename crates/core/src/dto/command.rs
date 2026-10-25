use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{error::ErrorInfo, settings::DeviceInfo};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandArgs {
    GetAccessToken,
    GetRefreshToken,
    SetAccessToken { token: String },
    SetRefreshToken { token: String },
    GetDeviceInfo,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowStartDragging,
    OpenDevTools,
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
