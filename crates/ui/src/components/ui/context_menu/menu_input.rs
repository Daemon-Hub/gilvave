//! Contextual menu items for text inputs, textareas, and contenteditable elements.

use std::rc::Rc;
use web_sys::Element;
use super::clipboard::{
    copy_to_clipboard, cut_from_target_element, select_all_in_target_element,
};
#[cfg(target_arch = "wasm32")]
use super::clipboard::paste_into_target_element;
use super::types::ContextMenuItem;
use crate::components::common::UiModalContext;

#[allow(dead_code)]
pub(crate) fn is_editable_element(el: &Element) -> bool {
    el.tag_name().eq_ignore_ascii_case("INPUT")
        || el.tag_name().eq_ignore_ascii_case("TEXTAREA")
        || el.get_attribute("contenteditable").as_deref() == Some("true")
        || el.closest("[contenteditable='true']").ok().flatten().is_some()
}

#[allow(dead_code)]
pub(crate) fn build_input_menu(
    el: &Element,
    selected_text: &str,
    modal_ctx: Option<&UiModalContext>,
) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();
    #[allow(unused_variables)]
    let modal_ctx_owned = modal_ctx.cloned();

    if !selected_text.is_empty() {
        let target_for_cut = el.clone();
        let modal_for_cut = modal_ctx.cloned();
        items.push(ContextMenuItem {
            label: "Вырезать".to_string(),
            icon: "cut",
            shortcut: Some("Ctrl+X"),
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || {
                cut_from_target_element(&target_for_cut, modal_for_cut.as_ref());
            }),
        });

        let sel = selected_text.to_string();
        items.push(ContextMenuItem {
            label: "Копировать".to_string(),
            icon: "copy",
            shortcut: Some("Ctrl+C"),
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || copy_to_clipboard(&sel)),
        });
    }

    // Вставить (Paste) — всегда доступно для полей ввода
    #[allow(unused_variables)]
    let target_clone = el.clone();
    items.push(ContextMenuItem {
        label: "Вставить".to_string(),
        icon: "paste",
        shortcut: Some("Ctrl+V"),
        danger: false,
        separator_above: false,
        on_click: Rc::new(move || {
            #[cfg(target_arch = "wasm32")]
            {
                let el_to_paste = target_clone.clone();
                let m_ctx = modal_ctx_owned.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(window) = web_sys::window() {
                        let clipboard = window.navigator().clipboard();
                        let promise = clipboard.read_text();
                        match wasm_bindgen_futures::JsFuture::from(promise).await {
                            Ok(js_val) => {
                                if let Some(text) = js_val.as_string() {
                                    paste_into_target_element(&el_to_paste, &text, m_ctx.as_ref());
                                }
                            }
                            Err(err) => {
                                web_sys::console::warn_1(
                                    &format!("[ContextMenu] Paste error: {err:?}").into(),
                                );
                            }
                        }
                    }
                });
            }
        }),
    });

    let target_for_sel = el.clone();
    items.push(ContextMenuItem {
        label: "Выделить всё".to_string(),
        icon: "select_all",
        shortcut: Some("Ctrl+A"),
        danger: false,
        separator_above: true,
        on_click: Rc::new(move || {
            select_all_in_target_element(&target_for_sel);
        }),
    });

    items
}
