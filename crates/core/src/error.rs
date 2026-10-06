use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorInfo(pub u16, pub String);

impl ErrorInfo {
    pub fn default(error: String) -> Self {
        // StatusCode::SERVICE_UNAVAILABLE
        Self(503u16, error)
    }
}
