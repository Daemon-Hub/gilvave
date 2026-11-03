//! Global context menu dispatcher identifying target element and opening the context menu.

use std::rc::Rc;
#[allow(unused_imports)]
use wasm_bindgen::JsCast;

#[allow(unused_imports)]
use super::clipboard::{copy_to_clipboard, get_window_selection_text};
#[cfg(target_arch = "wasm32")]
use super::menu_input::{build_input_menu, is_editable_element};
#[cfg(target_arch = "wasm32")]
use super::menu_message::build_message_menu;
#[cfg(target_arch = "wasm32")]
use super::menu_server_channel::{build_channel_menu, build_server_menu};
#[cfg(target_arch = "wasm32")]
use super::menu_user::build_user_menu;
use super::types::{ContextMenuContext, ContextMenuItem};
use crate::components::common::{
    ChannelContext, ServerContext, UiModalContext, UserProfileContext,
};

pub fn handle_global_context_menu(
    event: &web_sys::MouseEvent,
    menu_ctx: &ContextMenuContext,
    modal_ctx: Option<&UiModalContext>,
    #[allow(unused_variables)] channel_ctx: Option<&ChannelContext>,
    #[allow(unused_variables)] server_ctx: Option<&ServerContext>,
    #[allow(unused_variables)] user_ctx: Option<&UserProfileContext>,
) {
    event.prevent_default();
    let x = event.client_x();
    let y = event.client_y();

    let selected_text = get_window_selection_text();

    #[cfg(target_arch = "wasm32")]
    if let Some(target) = event.target() {
        if let Ok(el) = target.dyn_into::<web_sys::Element>() {
            // 1. Check if clicked inside an editable input / contenteditable / textarea
            if is_editable_element(&el) {
                let items = build_input_menu(&el, &selected_text, modal_ctx);
                menu_ctx.show(x, y, items);
                return;
            }

            // 2. Check if clicked on a user element (member item, avatar, header, friend row)
            if let Ok(Some(user_el)) = el.closest("[data-user-id]") {
                if let Some(items) = build_user_menu(
                    &el,
                    &user_el,
                    modal_ctx,
                    channel_ctx,
                    server_ctx,
                    user_ctx,
                ) {
                    menu_ctx.show(x, y, items);
                    return;
                }
            }

            // 3. Check if clicked inside a message item (message bubble / text)
            if let Ok(Some(msg_el)) = el.closest("[data-message-id]") {
                let items = build_message_menu(
                    &msg_el,
                    &selected_text,
                    modal_ctx,
                    channel_ctx,
                    server_ctx,
                    user_ctx,
                );
                menu_ctx.show(x, y, items);
                return;
            }

            // 4. Check if clicked on a channel item
            if let Ok(Some(ch_el)) = el.closest("[data-channel-id]") {
                let items = build_channel_menu(&ch_el);
                menu_ctx.show(x, y, items);
                return;
            }

            // 5. Check if clicked on a server icon
            if let Ok(Some(s_el)) = el.closest("[data-server-id]") {
                let items = build_server_menu(&s_el, modal_ctx, server_ctx, user_ctx);
                menu_ctx.show(x, y, items);
                return;
            }
        }
    }

    // 6. Active text selection anywhere
    if !selected_text.is_empty() {
        let sel = selected_text;
        let items = vec![ContextMenuItem {
            label: "Копировать выделенное".to_string(),
            icon: "copy",
            shortcut: Some("Ctrl+C"),
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || copy_to_clipboard(&sel)),
        }];
        menu_ctx.show(x, y, items);
        return;
    }

    // 7. Default general application menu
    let mut items = Vec::new();
    if let Some(m_ctx) = modal_ctx {
        let open_profile = m_ctx.is_profile_settings_open;
        items.push(ContextMenuItem {
            label: "Настройки профиля".to_string(),
            icon: "settings",
            shortcut: None,
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || open_profile.set(true)),
        });

        let open_catalog = m_ctx.is_theme_catalog_open;
        items.push(ContextMenuItem {
            label: "Каталог тем".to_string(),
            icon: "theme",
            shortcut: None,
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || open_catalog.set(true)),
        });
    }

    items.push(ContextMenuItem {
        label: "Инструменты разработчика".to_string(),
        icon: "devtools",
        shortcut: None,
        danger: false,
        separator_above: true,
        on_click: Rc::new(|| {
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(async {
                use gilvave_core::dto::command::CommandArgs;
                let _ = crate::utils::invoke_command(CommandArgs::OpenDevTools.to_json()).await;
            });
        }),
    });

    items.push(ContextMenuItem {
        label: "Перезагрузить приложение".to_string(),
        icon: "refresh",
        shortcut: Some("Ctrl+R"),
        danger: false,
        separator_above: true,
        on_click: Rc::new(|| {
            let _ = js_sys::eval("window.location.reload()");
        }),
    });

    menu_ctx.show(x, y, items);
}
