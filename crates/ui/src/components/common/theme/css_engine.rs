//! CSS Custom Properties generator, color mix utilities, security sanitization, and DOM style injection.

use wasm_bindgen::JsCast;

pub(crate) fn sanitize_id(input: &str) -> String {
    input
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(|c| c == '-' || c == '_')
        .to_string()
}

pub(crate) fn is_allowed_css_var(key: &str) -> bool {
    let valid_prefix = key.starts_with("--color-")
        || key.starts_with("--gradient-")
        || key.starts_with("--shadow-")
        || key.starts_with("--filter-");
    valid_prefix && key.len() <= 64 && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub(crate) fn is_safe_css_value(val: &str) -> bool {
    if val.is_empty() || val.len() > 400 {
        return false;
    }
    let lower = val.to_ascii_lowercase();
    !val.contains('{')
        && !val.contains('}')
        && !val.contains(';')
        && !val.contains('<')
        && !val.contains('>')
        && !val.contains('\\')
        && !lower.contains("url(")
        && !lower.contains("expression(")
        && !lower.contains("javascript:")
}

pub(crate) fn parse_hex_rgb(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.trim().strip_prefix('#')?;
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b))
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

pub(crate) fn with_alpha(color: &str, alpha: f32) -> String {
    if let Some((r, g, b)) = parse_hex_rgb(color) {
        format!("rgba({r}, {g}, {b}, {alpha:.2})")
    } else {
        let pct = (alpha * 100.0).round() as u32;
        format!("color-mix(in srgb, {color} {pct}%, transparent)")
    }
}

pub(crate) fn darken_color(color: &str, factor: f32) -> String {
    if let Some((r, g, b)) = parse_hex_rgb(color) {
        let dr = (r as f32 * factor).round() as u8;
        let dg = (g as f32 * factor).round() as u8;
        let db = (b as f32 * factor).round() as u8;
        format!("#{dr:02x}{dg:02x}{db:02x}")
    } else {
        let pct = (factor * 100.0).round() as u32;
        format!("color-mix(in srgb, {color} {pct}%, black)")
    }
}

pub(crate) fn set_style_tag(doc: &web_sys::Document, id: &str, css: &str) {
    if let Some(existing) = doc.get_element_by_id(id) {
        existing.set_text_content(Some(css));
        return;
    }
    if css.is_empty() {
        return;
    }
    if let Ok(style_el) = doc.create_element("style") {
        let _ = style_el.set_attribute("id", id);
        style_el.set_text_content(Some(css));
        if let Some(root) = doc.document_element() {
            let _ = root.append_child(&style_el);
        }
    }
}

pub fn download_json_file(filename: &str, content: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(doc) = window.document() else {
        return;
    };
    let encoded = js_sys::encode_uri_component(content);
    let data_url = format!("data:application/json;charset=utf-8,{encoded}");
    if let Ok(el) = doc.create_element("a") {
        let _ = el.set_attribute("href", &data_url);
        let _ = el.set_attribute("download", filename);
        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
            html_el.click();
        }
    }
}


