//! LocalStorage persistence and synchronization for owned server IDs.

use std::collections::HashSet;
use gilvave_core::ids::ServerId;

#[allow(dead_code)]
pub const OWNED_SERVERS_KEY: &str = "gilvave_owned_servers";

#[cfg(target_arch = "wasm32")]
pub fn get_owned_server_ids() -> HashSet<ServerId> {
    let mut set = HashSet::new();
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(json)) = storage.get_item(OWNED_SERVERS_KEY) {
            if let Ok(ids) = serde_json::from_str::<Vec<ServerId>>(&json) {
                set.extend(ids);
            }
        }
    }
    set
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn get_owned_server_ids() -> HashSet<ServerId> {
    HashSet::new()
}

#[cfg(target_arch = "wasm32")]
pub fn add_owned_server_id(id: ServerId) {
    let mut ids = get_owned_server_ids();
    if ids.insert(id) {
        save_owned_server_ids(&ids);
        if let Some(window) = web_sys::window() {
            if let Ok(ev) = web_sys::Event::new("gilvave_owned_servers_changed") {
                let _ = window.dispatch_event(&ev);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn add_owned_server_id(_id: ServerId) {}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub fn toggle_owned_server_id(id: ServerId) -> bool {
    let mut ids = get_owned_server_ids();
    let is_owned = if ids.contains(&id) {
        ids.remove(&id);
        false
    } else {
        ids.insert(id);
        true
    };
    save_owned_server_ids(&ids);
    if let Some(window) = web_sys::window() {
        if let Ok(ev) = web_sys::Event::new("gilvave_owned_servers_changed") {
            let _ = window.dispatch_event(&ev);
        }
    }
    is_owned
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn toggle_owned_server_id(_id: ServerId) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
pub fn save_owned_server_ids(ids: &HashSet<ServerId>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let vec: Vec<ServerId> = ids.iter().copied().collect();
        if let Ok(json) = serde_json::to_string(&vec) {
            let _ = storage.set_item(OWNED_SERVERS_KEY, &json);
        }
    }
}
