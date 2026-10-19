use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub error: String,
}

/// Logical category of the error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    Command,
    Network,
    Gateway,
    Platform,
    Auth,
    Http,
    Unknown,
}

/// Structured, grouped application error codes.
///
/// Ranges:
/// - 400..=599: Standard HTTP status codes
/// - 1000..=1999: Command & Serialization (IPC / Web Bridge)
/// - 2000..=2999: Network & HTTP Client (WASM fetch, DOM, responses)
/// - 3000..=3999: Gateway & WebSocket Client
/// - 4000..=4999: Platform & OS (Tauri, Keyring, Window controls)
/// - 5000..=5999: Auth & Security (Tokens, Storage, Sessions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum ErrorCode {
    // ── HTTP Status Codes ──
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    Conflict = 409,
    InternalServerError = 500,
    ServiceUnavailable = 503,

    // ── Group 1000..1999: Command & Serialization ──
    CommandError = 1000,
    CommandDeserializationFailed = 1001,
    CommandSerializationFailed = 1002,
    CommandUnknown = 1003,
    CommandExecutionFailed = 1004,
    CommandUnsupported = 1005,

    // ── Group 2000..2999: Network & HTTP Client ──
    NetworkError = 2000,
    WindowUnavailable = 2001,
    HttpRequestFailed = 2002,
    HttpInvalidHeaders = 2003,
    HttpInvalidResponse = 2004,
    HttpBodyReadFailed = 2005,
    HttpJsonDecodeFailed = 2006,
    HttpJsonEncodeFailed = 2007,

    // ── Group 3000..3999: Gateway & WebSocket ──
    GatewayError = 3000,
    GatewayNotInitialized = 3001,
    GatewayConnectionFailed = 3002,
    GatewaySendFailed = 3003,
    GatewayDisconnected = 3004,

    // ── Group 4000..4999: Platform & Native Controls ──
    PlatformError = 4000,
    KeyringAccessFailed = 4001,
    WindowControlFailed = 4002,
    DeviceInfoFailed = 4003,

    // ── Group 5000..5999: Auth & Security ──
    AuthError = 5000,
    TokenNotFound = 5001,
    TokenStorageFailed = 5002,
    SessionExpired = 5003,
}

impl ErrorCode {
    #[inline]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    pub const fn category(self) -> ErrorCategory {
        match self as u16 {
            100..=599 => ErrorCategory::Http,
            1000..=1999 => ErrorCategory::Command,
            2000..=2999 => ErrorCategory::Network,
            3000..=3999 => ErrorCategory::Gateway,
            4000..=4999 => ErrorCategory::Platform,
            5000..=5999 => ErrorCategory::Auth,
            _ => ErrorCategory::Unknown,
        }
    }

    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            400 => Some(Self::BadRequest),
            401 => Some(Self::Unauthorized),
            403 => Some(Self::Forbidden),
            404 => Some(Self::NotFound),
            409 => Some(Self::Conflict),
            500 => Some(Self::InternalServerError),
            503 => Some(Self::ServiceUnavailable),

            1000 => Some(Self::CommandError),
            1001 => Some(Self::CommandDeserializationFailed),
            1002 => Some(Self::CommandSerializationFailed),
            1003 => Some(Self::CommandUnknown),
            1004 => Some(Self::CommandExecutionFailed),
            1005 => Some(Self::CommandUnsupported),

            2000 => Some(Self::NetworkError),
            2001 => Some(Self::WindowUnavailable),
            2002 => Some(Self::HttpRequestFailed),
            2003 => Some(Self::HttpInvalidHeaders),
            2004 => Some(Self::HttpInvalidResponse),
            2005 => Some(Self::HttpBodyReadFailed),
            2006 => Some(Self::HttpJsonDecodeFailed),
            2007 => Some(Self::HttpJsonEncodeFailed),

            3000 => Some(Self::GatewayError),
            3001 => Some(Self::GatewayNotInitialized),
            3002 => Some(Self::GatewayConnectionFailed),
            3003 => Some(Self::GatewaySendFailed),
            3004 => Some(Self::GatewayDisconnected),

            4000 => Some(Self::PlatformError),
            4001 => Some(Self::KeyringAccessFailed),
            4002 => Some(Self::WindowControlFailed),
            4003 => Some(Self::DeviceInfoFailed),

            5000 => Some(Self::AuthError),
            5001 => Some(Self::TokenNotFound),
            5002 => Some(Self::TokenStorageFailed),
            5003 => Some(Self::SessionExpired),

            _ => None,
        }
    }
}

impl From<ErrorCode> for u16 {
    #[inline]
    fn from(code: ErrorCode) -> Self {
        code.as_u16()
    }
}

impl TryFrom<u16> for ErrorCode {
    type Error = u16;

    #[inline]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::from_u16(value).ok_or(value)
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({})", self, *self as u16)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorInfo(pub u16, pub String);

impl ErrorInfo {
    #[inline]
    pub fn new(code: impl Into<u16>, message: impl Into<String>) -> Self {
        Self(code.into(), message.into())
    }

    #[inline]
    pub fn from_code(code: ErrorCode, message: impl Into<String>) -> Self {
        Self(code.as_u16(), message.into())
    }

    #[inline]
    pub fn default(error: String) -> Self {
        Self(ErrorCode::ServiceUnavailable.as_u16(), error)
    }

    #[inline]
    pub fn code(&self) -> u16 {
        self.0
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.1
    }

    pub fn known_code(&self) -> Option<ErrorCode> {
        ErrorCode::from_u16(self.0)
    }

    pub fn category(&self) -> ErrorCategory {
        self.known_code()
            .map(|c| c.category())
            .unwrap_or_else(|| match self.0 {
                100..=599 => ErrorCategory::Http,
                1000..=1999 => ErrorCategory::Command,
                2000..=2999 => ErrorCategory::Network,
                3000..=3999 => ErrorCategory::Gateway,
                4000..=4999 => ErrorCategory::Platform,
                5000..=5999 => ErrorCategory::Auth,
                _ => ErrorCategory::Unknown,
            })
    }
}

impl std::fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error {}] {}", self.0, self.1)
    }
}

impl std::error::Error for ErrorInfo {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_conversions_and_categories() {
        assert_eq!(ErrorCode::CommandDeserializationFailed.as_u16(), 1001);
        assert_eq!(u16::from(ErrorCode::CommandDeserializationFailed), 1001);
        assert_eq!(
            ErrorCode::try_from(1001),
            Ok(ErrorCode::CommandDeserializationFailed)
        );
        assert_eq!(
            ErrorCode::CommandDeserializationFailed.category(),
            ErrorCategory::Command
        );

        assert_eq!(
            ErrorCode::GatewayNotInitialized.category(),
            ErrorCategory::Gateway
        );
        assert_eq!(
            ErrorCode::HttpRequestFailed.category(),
            ErrorCategory::Network
        );
        assert_eq!(
            ErrorCode::KeyringAccessFailed.category(),
            ErrorCategory::Platform
        );
        assert_eq!(ErrorCode::TokenNotFound.category(), ErrorCategory::Auth);
        assert_eq!(ErrorCode::NotFound.category(), ErrorCategory::Http);
    }

    #[test]
    fn test_error_info_constructors() {
        let err = ErrorInfo::from_code(
            ErrorCode::CommandDeserializationFailed,
            "Failed to parse payload",
        );
        assert_eq!(err.code(), 1001);
        assert_eq!(err.message(), "Failed to parse payload");
        assert_eq!(
            err.known_code(),
            Some(ErrorCode::CommandDeserializationFailed)
        );
        assert_eq!(err.category(), ErrorCategory::Command);
        assert_eq!(err.to_string(), "[Error 1001] Failed to parse payload");

        let custom_err = ErrorInfo::new(404u16, "Not found");
        assert_eq!(custom_err.code(), 404);
        assert_eq!(custom_err.category(), ErrorCategory::Http);
    }
}
