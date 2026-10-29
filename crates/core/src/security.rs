#[cfg(not(target_os = "android"))]
use keyring::Entry;

#[cfg(not(target_os = "android"))]
fn get_keyring(key: &str) -> Entry {
    Entry::new("gilvave-app", key).unwrap()
}

#[cfg(not(target_os = "android"))]
pub fn get_access_token() -> String {
    get_keyring("access_token")
        .get_password()
        .unwrap_or_default()
}

#[cfg(not(target_os = "android"))]
pub fn get_refresh_token() -> String {
    get_keyring("refresh_token")
        .get_password()
        .unwrap_or_default()
}

#[cfg(not(target_os = "android"))]
pub fn set_access_token(token: &str) {
    get_keyring("access_token").set_password(token).ok();
}

#[cfg(not(target_os = "android"))]
pub fn set_refresh_token(token: &str) {
    get_keyring("refresh_token").set_password(token).ok();
}

// In-memory token storage for Android platform
#[cfg(target_os = "android")]
use std::sync::RwLock;

#[cfg(target_os = "android")]
static ACCESS_TOKEN: RwLock<String> = RwLock::new(String::new());

#[cfg(target_os = "android")]
static REFRESH_TOKEN: RwLock<String> = RwLock::new(String::new());

#[cfg(target_os = "android")]
pub fn get_access_token() -> String {
    ACCESS_TOKEN.read().map(|t| t.clone()).unwrap_or_default()
}

#[cfg(target_os = "android")]
pub fn get_refresh_token() -> String {
    REFRESH_TOKEN.read().map(|t| t.clone()).unwrap_or_default()
}

#[cfg(target_os = "android")]
pub fn set_access_token(token: &str) {
    if let Ok(mut lock) = ACCESS_TOKEN.write() {
        *lock = token.to_string();
    }
}

#[cfg(target_os = "android")]
pub fn set_refresh_token(token: &str) {
    if let Ok(mut lock) = REFRESH_TOKEN.write() {
        *lock = token.to_string();
    }
}
