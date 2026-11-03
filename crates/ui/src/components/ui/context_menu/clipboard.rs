//! Clipboard interactions, selection helpers, and toast dispatchers.

use crate::components::common::UiModalContext;

pub fn copy_to_clipboard(text: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let nav = window.navigator();
        let clipboard = nav.clipboard();
        let _ = clipboard.write_text(text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = text;
}

pub fn get_window_selection_text() -> String {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(selection)) = window.get_selection() {
            let js_str = selection.to_string();
            let text = String::from(js_str);
            return text.trim().to_string();
        }
    }
    String::new()
}

#[cfg(target_arch = "wasm32")]
pub fn paste_into_target_element(
    el: &web_sys::Element,
    text: &str,
    modal_ctx: Option<&UiModalContext>,
) {
    use wasm_bindgen::JsCast;
    if let Ok(input) = el.clone().dyn_into::<web_sys::HtmlInputElement>() {
        let val = input.value();
        let start = input.selection_start().ok().flatten().unwrap_or(val.len() as u32) as usize;
        let end = input.selection_end().ok().flatten().unwrap_or(val.len() as u32) as usize;
        let mut new_val = String::with_capacity(val.len() + text.len());
        let start_clamped = start.min(val.len());
        let end_clamped = end.min(val.len()).max(start_clamped);
        new_val.push_str(&val[..start_clamped]);
        new_val.push_str(text);
        new_val.push_str(&val[end_clamped..]);
        input.set_value(&new_val);
        let new_pos = (start_clamped + text.len()) as u32;
        let _ = input.set_selection_range(new_pos, new_pos);
        let _ = input.dispatch_event(&web_sys::Event::new("input").unwrap());
    } else if let Ok(textarea) = el.clone().dyn_into::<web_sys::HtmlTextAreaElement>() {
        let val = textarea.value();
        let start = textarea.selection_start().ok().flatten().unwrap_or(val.len() as u32) as usize;
        let end = textarea.selection_end().ok().flatten().unwrap_or(val.len() as u32) as usize;
        let mut new_val = String::with_capacity(val.len() + text.len());
        let start_clamped = start.min(val.len());
        let end_clamped = end.min(val.len()).max(start_clamped);
        new_val.push_str(&val[..start_clamped]);
        new_val.push_str(text);
        new_val.push_str(&val[end_clamped..]);
        textarea.set_value(&new_val);
        let new_pos = (start_clamped + text.len()) as u32;
        let _ = textarea.set_selection_range(new_pos, new_pos);
        let _ = textarea.dispatch_event(&web_sys::Event::new("input").unwrap());
    } else {
        let target_editor = el
            .closest("[contenteditable='true']")
            .ok()
            .flatten()
            .unwrap_or_else(|| el.clone());
        crate::components::common::insert_text_into_editor(&target_editor, text);
        let serialized =
            crate::components::common::serialize_contenteditable(target_editor.as_ref());
        if serialized.trim().is_empty() {
            let _ = target_editor.set_attribute("data-empty", "true");
        } else {
            let _ = target_editor.remove_attribute("data-empty");
        }
        if let Some(m_ctx) = modal_ctx {
            m_ctx.draft_message.set(serialized);
        }
        let _ = target_editor.dispatch_event(&web_sys::Event::new("input").unwrap());
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn paste_into_target_element(
    _el: &web_sys::Element,
    _text: &str,
    _modal_ctx: Option<&UiModalContext>,
) {
}

#[cfg(target_arch = "wasm32")]
pub fn cut_from_target_element(el: &web_sys::Element, modal_ctx: Option<&UiModalContext>) {
    use wasm_bindgen::JsCast;
    if let Ok(input) = el.clone().dyn_into::<web_sys::HtmlInputElement>() {
        let val = input.value();
        let start = input.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = input.selection_end().ok().flatten().unwrap_or(0) as usize;
        if start < end {
            let sel_text = &val[start..end.min(val.len())];
            copy_to_clipboard(sel_text);
            let mut new_val = String::new();
            new_val.push_str(&val[..start]);
            if end < val.len() {
                new_val.push_str(&val[end..]);
            }
            input.set_value(&new_val);
            let _ = input.set_selection_range(start as u32, start as u32);
            let _ = input.dispatch_event(&web_sys::Event::new("input").unwrap());
        }
    } else if let Ok(textarea) = el.clone().dyn_into::<web_sys::HtmlTextAreaElement>() {
        let val = textarea.value();
        let start = textarea.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = textarea.selection_end().ok().flatten().unwrap_or(0) as usize;
        if start < end {
            let sel_text = &val[start..end.min(val.len())];
            copy_to_clipboard(sel_text);
            let mut new_val = String::new();
            new_val.push_str(&val[..start]);
            if end < val.len() {
                new_val.push_str(&val[end..]);
            }
            textarea.set_value(&new_val);
            let _ = textarea.set_selection_range(start as u32, start as u32);
            let _ = textarea.dispatch_event(&web_sys::Event::new("input").unwrap());
        }
    } else {
        let sel_text = get_window_selection_text();
        if !sel_text.is_empty() {
            copy_to_clipboard(&sel_text);
            if let Some(window) = web_sys::window() {
                if let Ok(Some(selection)) = window.get_selection() {
                    let _ = selection.delete_from_document();
                }
            }
            let target_editor = el
                .closest("[contenteditable='true']")
                .ok()
                .flatten()
                .unwrap_or_else(|| el.clone());
            let serialized =
                crate::components::common::serialize_contenteditable(target_editor.as_ref());
            if serialized.trim().is_empty() {
                let _ = target_editor.set_attribute("data-empty", "true");
            } else {
                let _ = target_editor.remove_attribute("data-empty");
            }
            if let Some(m_ctx) = modal_ctx {
                m_ctx.draft_message.set(serialized);
            }
            let _ = target_editor.dispatch_event(&web_sys::Event::new("input").unwrap());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn cut_from_target_element(_el: &web_sys::Element, _modal_ctx: Option<&UiModalContext>) {}

#[cfg(target_arch = "wasm32")]
pub fn select_all_in_target_element(el: &web_sys::Element) {
    use wasm_bindgen::JsCast;
    if let Ok(input) = el.clone().dyn_into::<web_sys::HtmlInputElement>() {
        let _ = input.select();
    } else if let Ok(textarea) = el.clone().dyn_into::<web_sys::HtmlTextAreaElement>() {
        let _ = textarea.select();
    } else {
        let target_editor = el
            .closest("[contenteditable='true']")
            .ok()
            .flatten()
            .unwrap_or_else(|| el.clone());
        if let Some(window) = web_sys::window() {
            if let Ok(Some(selection)) = window.get_selection() {
                let _ = selection.select_all_children(&target_editor);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn select_all_in_target_element(_el: &web_sys::Element) {}

#[allow(dead_code, unused_variables)]
pub fn show_toast(modal_ctx: &UiModalContext, is_err: bool, msg: String) {
    let toast_sig = modal_ctx.toast_message;
    toast_sig.set(Some((is_err, msg)));
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(4000).await;
        toast_sig.set(None);
    });
}


