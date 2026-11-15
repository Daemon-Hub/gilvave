use wasm_bindgen::JsCast;

/// Читает значение cookie по имени через `document.cookie`.
fn cookie_get(name: &str) -> String {
    let doc = match web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        Some(d) => d,
        None => return String::new(),
    };
    let all = match doc.cookie() {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    for pair in all.split(';') {
        let pair = pair.trim();
        if let Some(rest) = pair.strip_prefix(name)
            && let Some(val) = rest.strip_prefix('=')
        {
            return val.to_string();
        }
    }
    String::new()
}

/// Записывает cookie через `document.cookie`.
fn cookie_set(name: &str, value: &str) {
    let doc = match web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        Some(d) => d,
        None => return,
    };
    let cookie_str = if value.is_empty() {
        format!("{name}=; path=/; Max-Age=0; SameSite=Strict")
    } else {
        format!("{name}={value}; path=/; SameSite=Strict")
    };
    let _ = doc.set_cookie(&cookie_str);
}

pub fn get_access_token() -> String {
    cookie_get("access_token")
}

pub fn get_refresh_token() -> String {
    cookie_get("refresh_token")
}

pub fn set_access_token(token: &str) {
    cookie_set("access_token", token);
}

pub fn set_refresh_token(token: &str) {
    cookie_set("refresh_token", token);
}
