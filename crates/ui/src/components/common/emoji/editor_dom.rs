//! DOM manipulation and contenteditable editor synchronization for inline emojis and rich text.

#[cfg(target_arch = "wasm32")]
use super::twemoji::{parse_message_segments_default, MessageSegment};

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
                    if let Some(pack) = el.get_attribute("data-pack") {
                        if !pack.is_empty() {
                            use std::fmt::Write;
                            let _ = write!(out, "<e:{pack}:{emoji}>");
                            return;
                        }
                    }
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
    pack_id: &str,
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
    let _ = img.set_attribute("data-pack", pack_id);
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
    if let Ok(Some(selection)) = window.get_selection() {
        if let Ok(range) = document.create_range() {
            let _ = range.set_start_after(&img);
            let _ = range.set_end_after(&img);
            let _ = selection.remove_all_ranges();
            let _ = selection.add_range(&range);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn insert_emoji_into_editor(
    _editor: &web_sys::Element,
    _emoji_char: &str,
    _emoji_url: &str,
    _emoji_title: &str,
    _pack_id: &str,
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

    let segments = parse_message_segments_default(text);
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
            MessageSegment::Emoji {
                char,
                url,
                name,
                pack,
            } => {
                if let Ok(img_node) = document.create_element("img") {
                    if let Ok(img) = img_node.dyn_into::<web_sys::HtmlImageElement>() {
                        img.set_class_name("chat-inline-emoji");
                        img.set_src(&url);
                        let _ = img.set_attribute("alt", &char);
                        let _ = img.set_attribute("data-emoji", &char);
                        if let Some(p) = pack {
                            let _ = img.set_attribute("data-pack", p.as_str());
                        }
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

#[cfg(target_arch = "wasm32")]
pub fn focus_editor_at_end(editor: &web_sys::HtmlElement) {
    let _ = editor.focus();
    if let Some(window) = web_sys::window() {
        if let Ok(Some(selection)) = window.get_selection() {
            if let Some(doc) = window.document() {
                if let Ok(range) = doc.create_range() {
                    let _ = range.select_node_contents(editor);
                    let _ = range.collapse_with_to_start(false);
                    let _ = selection.remove_all_ranges();
                    let _ = selection.add_range(&range);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn focus_editor_at_end(_editor: &web_sys::HtmlElement) {}
