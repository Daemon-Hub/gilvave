//! Twemoji URL generator and emoji text parser for cross-platform emoji rendering.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSegment {
    Text(String),
    Emoji {
        char: String,
        url: String,
        name: String,
    },
}

/// Converts a Unicode emoji sequence into its corresponding Twemoji SVG CDN URL.
///
/// Follows the official Twemoji naming specification:
/// - If the emoji does not contain Zero Width Joiner (`\u{200D}`), strip all Variation Selector 16 (`\u{FE0F}`) characters.
/// - If the emoji contains `\u{200D}`, keep Variation Selector 16.
/// - Convert each code point to lowercase hex and join with `-`.
pub fn emoji_to_twemoji_url(emoji: &str) -> String {
    let has_zwj = emoji.contains('\u{200D}');
    let mut parts = Vec::new();
    for ch in emoji.chars() {
        if !has_zwj && ch == '\u{FE0F}' {
            continue;
        }
        parts.push(format!("{:x}", ch as u32));
    }
    format!(
        "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/{}.svg",
        parts.join("-")
    )
}

/// Parses message text into plain text chunks and recognized emoji segments.
/// Emojis are matched greedily using the `emojis` crate catalog.
pub fn parse_message_segments(text: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let first_char = remaining.chars().next().unwrap();

        // Fast-path filter: skip characters that cannot start any valid emoji
        let is_candidate = matches!(
            first_char,
            '#' | '*'
                | '0'..='9'
                | '\u{2000}'..='\u{3299}'
                | '\u{1F000}'..='\u{1FAFF}'
        );

        if !is_candidate {
            current_text.push(first_char);
            remaining = &remaining[first_char.len_utf8()..];
            continue;
        }

        // Greedy matching: find the longest matching emoji sequence starting here
        let mut matched: Option<(&'static emojis::Emoji, usize)> = None;
        let mut byte_len = 0;
        let mut char_count = 0;

        for ch in remaining.chars() {
            byte_len += ch.len_utf8();
            char_count += 1;
            if byte_len > 32 || char_count > 12 {
                break;
            }
            if let Some(emoji) = emojis::get(&remaining[..byte_len]) {
                matched = Some((emoji, byte_len));
            }
        }

        if let Some((emoji, len)) = matched {
            if !current_text.is_empty() {
                segments.push(MessageSegment::Text(std::mem::take(&mut current_text)));
            }
            let emoji_str = emoji.as_str().to_string();
            let url = emoji_to_twemoji_url(&emoji_str);
            segments.push(MessageSegment::Emoji {
                char: emoji_str,
                url,
                name: emoji.name().to_string(),
            });
            remaining = &remaining[len..];
        } else {
            current_text.push(first_char);
            remaining = &remaining[first_char.len_utf8()..];
        }
    }

    if !current_text.is_empty() {
        segments.push(MessageSegment::Text(current_text));
    }

    segments
}

#[cfg(target_arch = "wasm32")]
pub fn serialize_contenteditable(node: &web_sys::Node) -> String {
    let mut out = String::new();
    serialize_node_inner(node, &mut out);
    out
}

#[cfg(not(target_arch = "wasm32"))]
pub fn serialize_contenteditable(_node: &web_sys::Node) -> String {
    String::new()
}

#[cfg(target_arch = "wasm32")]
fn serialize_node_inner(node: &web_sys::Node, out: &mut String) {
    use wasm_bindgen::JsCast;

    let node_type = node.node_type();
    if node_type == web_sys::Node::TEXT_NODE {
        if let Some(val) = node.node_value() {
            out.push_str(&val);
        }
        return;
    }

    if node_type == web_sys::Node::ELEMENT_NODE {
        if let Ok(el) = node.clone().dyn_into::<web_sys::Element>() {
            let tag = el.tag_name().to_uppercase();
            if tag == "IMG" {
                if let Some(emoji) = el.get_attribute("data-emoji") {
                    out.push_str(&emoji);
                } else if let Some(alt) = el.get_attribute("alt") {
                    out.push_str(&alt);
                }
                return;
            } else if tag == "BR" {
                out.push('\n');
                return;
            } else if tag == "DIV" || tag == "P" {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
    }

    let children = node.child_nodes();
    let len = children.length();
    for i in 0..len {
        if let Some(child) = children.item(i) {
            serialize_node_inner(&child, out);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn insert_emoji_into_editor(
    editor: &web_sys::Element,
    emoji_char: &str,
    emoji_url: &str,
    emoji_title: &str,
) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    let Ok(img_node) = document.create_element("img") else {
        return;
    };
    let Ok(img) = img_node.dyn_into::<web_sys::HtmlImageElement>() else {
        return;
    };
    img.set_class_name("chat-inline-emoji");
    img.set_src(emoji_url);
    let _ = img.set_attribute("alt", emoji_char);
    let _ = img.set_attribute("data-emoji", emoji_char);
    let _ = img.set_attribute("title", emoji_title);
    let _ = img.set_attribute("draggable", "false");

    if let Ok(Some(selection)) = window.get_selection() {
        if selection.range_count() > 0 {
            if let Ok(range) = selection.get_range_at(0) {
                let mut common_parent = range.common_ancestor_container().ok();
                let mut is_inside = false;
                while let Some(parent) = common_parent {
                    if parent == *editor.as_ref() {
                        is_inside = true;
                        break;
                    }
                    common_parent = parent.parent_node();
                }

                if is_inside {
                    let _ = range.delete_contents();
                    if range.insert_node(&img).is_ok() {
                        let _ = range.set_start_after(&img);
                        let _ = range.set_end_after(&img);
                        selection.remove_all_ranges().ok();
                        selection.add_range(&range).ok();
                        return;
                    }
                }
            }
        }
    }

    let _ = editor.append_child(&img);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn insert_emoji_into_editor(
    _editor: &web_sys::Element,
    _emoji_char: &str,
    _emoji_url: &str,
    _emoji_title: &str,
) {
}

#[cfg(target_arch = "wasm32")]
pub fn insert_text_into_editor(editor: &web_sys::Element, text: &str) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    let segments = parse_message_segments(text);
    let fragment = document.create_document_fragment();

    for seg in segments {
        match seg {
            MessageSegment::Text(t) => {
                let lines: Vec<&str> = t.split('\n').collect();
                for (idx, line) in lines.iter().enumerate() {
                    if idx > 0 {
                        if let Ok(br) = document.create_element("br") {
                            let _ = fragment.append_child(&br);
                        }
                    }
                    if !line.is_empty() {
                        let text_node = document.create_text_node(line);
                        let _ = fragment.append_child(&text_node);
                    }
                }
            }
            MessageSegment::Emoji { char, url, name } => {
                if let Ok(img_node) = document.create_element("img") {
                    if let Ok(img) = img_node.dyn_into::<web_sys::HtmlImageElement>() {
                        img.set_class_name("chat-inline-emoji");
                        img.set_src(&url);
                        let _ = img.set_attribute("alt", &char);
                        let _ = img.set_attribute("data-emoji", &char);
                        let _ = img.set_attribute("title", &name);
                        let _ = img.set_attribute("draggable", "false");
                        let _ = fragment.append_child(&img);
                    }
                }
            }
        }
    }

    if let Ok(Some(selection)) = window.get_selection() {
        if selection.range_count() > 0 {
            if let Ok(range) = selection.get_range_at(0) {
                let mut common_parent = range.common_ancestor_container().ok();
                let mut is_inside = false;
                while let Some(parent) = common_parent {
                    if parent == *editor.as_ref() {
                        is_inside = true;
                        break;
                    }
                    common_parent = parent.parent_node();
                }

                if is_inside {
                    let _ = range.delete_contents();
                    if range.insert_node(&fragment).is_ok() {
                        range.collapse_with_to_start(false);
                        selection.remove_all_ranges().ok();
                        selection.add_range(&range).ok();
                        return;
                    }
                }
            }
        }
    }

    let _ = editor.append_child(&fragment);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn insert_text_into_editor(_editor: &web_sys::Element, _text: &str) {}

#[cfg(target_arch = "wasm32")]
pub fn sync_editor_content(editor: &web_sys::Element, text: &str) {
    let current = serialize_contenteditable(editor.as_ref());
    if current == text {
        return;
    }
    editor.set_inner_html("");
    if !text.is_empty() {
        insert_text_into_editor(editor, text);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_editor_content(_editor: &web_sys::Element, _text: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twemoji_url_generation() {
        // Flag (Regional Indicator pair)
        assert_eq!(
            emoji_to_twemoji_url("🇷🇺"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/1f1f7-1f1fa.svg"
        );
        // Simple emoji without VS16
        assert_eq!(
            emoji_to_twemoji_url("😀"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/1f600.svg"
        );
        // Emoji with stripped VS16
        assert_eq!(
            emoji_to_twemoji_url("❤️"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/2764.svg"
        );
        // Complex ZWJ sequence keeps VS16
        assert_eq!(
            emoji_to_twemoji_url("🏴‍☠️"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/1f3f4-200d-2620-fe0f.svg"
        );
        // Keycap without ZWJ strips VS16
        assert_eq!(
            emoji_to_twemoji_url("1️⃣"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/31-20e3.svg"
        );
    }

    #[test]
    fn test_parse_message_segments() {
        let msg = "Привет 🇷🇺! Как дела 🫠? Супер 👍 и 🏴‍☠️ ого! Цифра 42.";
        let segments = parse_message_segments(msg);

        // Check text and emoji separation
        assert_eq!(segments[0], MessageSegment::Text("Привет ".to_string()));
        assert!(matches!(&segments[1], MessageSegment::Emoji { char, .. } if char == "🇷🇺"));
        assert_eq!(segments[2], MessageSegment::Text("! Как дела ".to_string()));
        assert!(matches!(&segments[3], MessageSegment::Emoji { char, .. } if char == "🫠"));
        assert_eq!(segments[4], MessageSegment::Text("? Супер ".to_string()));
        assert!(matches!(&segments[5], MessageSegment::Emoji { char, .. } if char == "👍"));
        assert_eq!(segments[6], MessageSegment::Text(" и ".to_string()));
        assert!(matches!(&segments[7], MessageSegment::Emoji { char, .. } if char == "🏴‍☠️"));
        assert_eq!(
            segments[8],
            MessageSegment::Text(" ого! Цифра 42.".to_string())
        );
    }

    #[test]
    fn test_plain_text_without_emojis() {
        let msg = "Просто текст без смайлов 123.";
        let segments = parse_message_segments(msg);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], MessageSegment::Text(msg.to_string()));
    }
}
