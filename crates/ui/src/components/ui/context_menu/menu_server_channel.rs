//! Contextual menu items for server icons and channel tree items.

use std::rc::Rc;
use web_sys::Element;
use super::clipboard::copy_to_clipboard;
use super::types::ContextMenuItem;
use crate::components::common::{ServerContext, UiModalContext, UserProfileContext};

#[allow(dead_code)]
pub(crate) fn build_channel_menu(ch_el: &Element) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();
    let ch_id = ch_el.get_attribute("data-channel-id").unwrap_or_default();
    let ch_name = ch_el.get_attribute("data-channel-name").unwrap_or_default();

    let n_copy = ch_name.clone();
    items.push(ContextMenuItem {
        label: format!("Скопировать #{n_copy}"),
        icon: "channel",
        shortcut: None,
        danger: false,
        separator_above: false,
        on_click: Rc::new(move || copy_to_clipboard(&n_copy)),
    });

    let id_copy = ch_id;
    items.push(ContextMenuItem {
        label: "Скопировать ID канала".to_string(),
        icon: "info",
        shortcut: None,
        danger: false,
        separator_above: false,
        on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
    });

    items
}

#[allow(dead_code)]
pub(crate) fn build_server_menu(
    s_el: &Element,
    modal_ctx: Option<&UiModalContext>,
    server_ctx: Option<&ServerContext>,
    user_ctx: Option<&UserProfileContext>,
) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();
    let s_id = s_el.get_attribute("data-server-id").unwrap_or_default();

    let is_server_owner = if let Ok(s_id_typed) =
        gilvave_core::ids::ServerId::try_from(s_id.as_str())
    {
        let from_list = server_ctx.is_some_and(|sc| {
            sc.list
                .with(|list| list.iter().any(|s| s.id == s_id_typed && s.is_owner))
        });
        let from_curr = server_ctx.is_some_and(|sc| {
            sc.current.with(|curr| {
                curr.as_ref().is_some_and(|curr| {
                    curr.id == s_id_typed
                        && (user_ctx.and_then(|u| u.user_id.get()) == Some(curr.owner_id))
                })
            })
        });
        from_list || from_curr
    } else {
        false
    };

    if is_server_owner
        && let Some(m_ctx) = modal_ctx
    {
        let open_settings = m_ctx.is_server_settings_open;
        items.push(ContextMenuItem {
            label: "Настройки сервера".to_string(),
            icon: "settings",
            shortcut: None,
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || open_settings.set(true)),
        });
    }

    let id_copy = s_id;
    items.push(ContextMenuItem {
        label: "Скопировать ID сервера".to_string(),
        icon: "info",
        shortcut: None,
        danger: false,
        separator_above: false,
        on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
    });

    items
}
