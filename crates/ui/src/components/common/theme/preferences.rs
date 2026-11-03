//! Local workspace and UI preference flags persisted in localStorage.

use super::{
    STORAGE_KEY_CLOSE_TO_TRAY, STORAGE_KEY_SHOW_GREETING, STORAGE_KEY_SHOW_QUICK_ACTIONS,
    STORAGE_KEY_WINDOWED_MODE,
};


pub fn load_close_to_tray() -> bool {
    if crate::components::common::is_mobile_device() {
        return false;
    }
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY_CLOSE_TO_TRAY).ok().flatten())
        .map(|v| v != "false")
        .unwrap_or(true)
}

pub fn save_close_to_tray(enabled: bool) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item(
            STORAGE_KEY_CLOSE_TO_TRAY,
            if enabled { "true" } else { "false" },
        );
    }
}

pub fn load_show_greeting() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY_SHOW_GREETING).ok().flatten())
        .map(|v| v != "false")
        .unwrap_or(true)
}

pub fn save_show_greeting(show: bool) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item(
            STORAGE_KEY_SHOW_GREETING,
            if show { "true" } else { "false" },
        );
    }
}

pub fn load_show_quick_actions() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY_SHOW_QUICK_ACTIONS).ok().flatten())
        .map(|v| v != "false")
        .unwrap_or(true)
}

pub fn save_show_quick_actions(show: bool) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item(
            STORAGE_KEY_SHOW_QUICK_ACTIONS,
            if show { "true" } else { "false" },
        );
    }
}

pub fn load_windowed_mode() -> bool {
    if crate::components::common::is_mobile_device() {
        return false;
    }
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY_WINDOWED_MODE).ok().flatten())
        .map(|v| v != "false")
        .unwrap_or(true)
}

pub fn save_windowed_mode(is_windowed: bool) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item(
            STORAGE_KEY_WINDOWED_MODE,
            if is_windowed { "true" } else { "false" },
        );
    }
}



