//! Contextual menu items for user elements (avatars, member lists, friend rows).

use std::rc::Rc;
use web_sys::Element;
use gilvave_core::ids::{MessageId, UserId};
use super::clipboard::copy_to_clipboard;
#[cfg(target_arch = "wasm32")]
use super::clipboard::show_toast;
use super::types::ContextMenuItem;
use crate::components::common::{ChannelContext, ServerContext, UiModalContext, UserProfileContext};

#[allow(dead_code)]
pub(crate) fn build_user_menu(
    el: &Element,
    user_el: &Element,
    modal_ctx: Option<&UiModalContext>,
    channel_ctx: Option<&ChannelContext>,
    server_ctx: Option<&ServerContext>,
    user_ctx: Option<&UserProfileContext>,
) -> Option<Vec<ContextMenuItem>> {
    let u_id_str = user_el.get_attribute("data-user-id")?;
    let u_name = user_el.get_attribute("data-username").unwrap_or_default();
    let target_uid = UserId::try_from(u_id_str.as_str()).ok()?;

    let mut items = Vec::new();
    let is_self = user_ctx
        .and_then(|u| u.user_id.get()) == Some(target_uid);

    let is_friend_card = user_el.class_name().contains("friend-row-item")
        || user_el.closest(".friend-row-item").ok().flatten().is_some();

    let is_friend = is_friend_card
        || user_ctx.is_some_and(|u| {
            u.friends.with(|list| {
                list.iter().any(|f| {
                    f.id == target_uid
                        || (!u_name.is_empty() && f.username.eq_ignore_ascii_case(&u_name))
                })
            })
        });

    if !is_self {
        if !is_friend {
            #[allow(unused_variables)]
            let m_ctx_clone = modal_ctx.cloned();
            #[allow(unused_variables)]
            let name_clone = u_name.clone();
            items.push(ContextMenuItem {
                label: "Добавить в друзья".to_string(),
                icon: "user_plus",
                shortcut: None,
                danger: false,
                separator_above: false,
                on_click: Rc::new(move || {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let m_ctx = m_ctx_clone.clone();
                        let target_display = if !name_clone.is_empty() {
                            format!("@{name_clone}")
                        } else {
                            "пользователю".to_string()
                        };
                        wasm_bindgen_futures::spawn_local(async move {
                            use gilvave_core::dto::users::friends::SendFriendRequest;
                        let req = SendFriendRequest {
                            user_id: Some(target_uid),
                            username: None,
                        };
                        match crate::http::api::Api::send_friend_request(&req).await {
                            Ok(res) => {
                                if let Some(m) = m_ctx.as_ref() {
                                    show_toast(m, false, res.message);
                                }
                            }
                            Err(err) => {
                                if let Some(m) = m_ctx.as_ref() {
                                    show_toast(
                                        m,
                                        true,
                                        format!(
                                            "Не удалось отправить запрос {}: {}",
                                            target_display, err.1
                                        ),
                                    );
                                }
                            }
                        }
                    });
                    }
                }),
            });
        } else {
            if let Some(m_ctx) = modal_ctx {
                let dm_name_sig = m_ctx.selected_dm_name;
                let home_tab_sig = m_ctx.home_tab;
                let s_ctx = server_ctx.cloned();
                let name_for_dm = u_name.clone();
                items.push(ContextMenuItem {
                    label: "Написать сообщение".to_string(),
                    icon: "message",
                    shortcut: None,
                    danger: false,
                    separator_above: false,
                    on_click: Rc::new(move || {
                        if let Some(sc) = s_ctx.as_ref() {
                            sc.current.set(None);
                        }
                        if !name_for_dm.is_empty() {
                            dm_name_sig.set(Some(name_for_dm.clone()));
                        }
                        home_tab_sig.set(crate::components::common::HomeTab::Chats);
                    }),
                });
            }

            #[allow(unused_variables)]
            let m_ctx_clone = modal_ctx.cloned();
            #[allow(unused_variables)]
            let name_clone = u_name.clone();
            #[allow(unused_variables)]
            let u_ctx_clone = user_ctx.cloned();
            items.push(ContextMenuItem {
                label: "Удалить из друзей".to_string(),
                icon: "trash",
                shortcut: None,
                danger: true,
                separator_above: true,
                on_click: Rc::new(move || {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let m_ctx = m_ctx_clone.clone();
                        let target_display = if !name_clone.is_empty() {
                            format!("@{name_clone}")
                        } else {
                            "пользователь".to_string()
                        };
                        let u_ctx_clone = u_ctx_clone.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match crate::http::api::Api::remove_friend(target_uid).await {
                            Ok(()) => {
                                if let Some(u) = u_ctx_clone.as_ref() {
                                    u.friends.update(|list| list.retain(|f| f.id != target_uid));
                                }
                                if let Some(m) = m_ctx.as_ref() {
                                    show_toast(
                                        m,
                                        false,
                                        format!("Пользователь {} удален из друзей", target_display),
                                    );
                                }
                            }
                            Err(err) => {
                                if let Some(m) = m_ctx.as_ref() {
                                    show_toast(m, true, format!("Ошибка удаления: {}", err.1));
                                }
                            }
                        }
                    });
                    }
                }),
            });
        }
    }

    if !u_name.is_empty() {
        let n_copy = format!("@{u_name}");
        items.push(ContextMenuItem {
            label: "Копировать имя пользователя".to_string(),
            icon: "copy",
            shortcut: None,
            danger: false,
            separator_above: true,
            on_click: Rc::new(move || copy_to_clipboard(&n_copy)),
        });
    }

    let id_copy = target_uid.to_string();
    items.push(ContextMenuItem {
        label: "Копировать ID пользователя".to_string(),
        icon: "info",
        shortcut: None,
        danger: false,
        separator_above: u_name.is_empty(),
        on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
    });

    // If clicked on an author avatar/header inside a message, also offer "Ответить"
    if let Ok(Some(msg_el)) = el.closest("[data-message-id]")
        && let Some(m_ctx) = modal_ctx
    {
        let msg_id_str = msg_el.get_attribute("data-message-id").unwrap_or_default();
        let parsed_msg_id = MessageId::try_from(msg_id_str.as_str()).ok();
        let found_msg = channel_ctx.and_then(|c_ctx| {
            parsed_msg_id.and_then(|m_id| {
                c_ctx
                    .messages
                    .with(|msgs| msgs.iter().find(|m| m.id == m_id).cloned())
            })
        });
        if let Some(target_msg) = found_msg {
            let replying_sig = m_ctx.replying_to;
            let editing_sig = m_ctx.editing_message;
            items.push(ContextMenuItem {
                label: "Ответить на сообщение".to_string(),
                icon: "reply",
                shortcut: None,
                danger: false,
                separator_above: true,
                on_click: Rc::new(move || {
                    editing_sig.set(None);
                    replying_sig.set(Some(target_msg.clone()));
                }),
            });
        }
    }

    Some(items)
}
