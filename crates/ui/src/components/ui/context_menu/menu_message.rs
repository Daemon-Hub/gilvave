//! Contextual menu items for message bubbles (reply, edit, forward, delete, copy, user actions).

use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use web_sys::Element;

use super::clipboard::copy_to_clipboard;
#[cfg(target_arch = "wasm32")]
use super::clipboard::show_toast;
use super::types::ContextMenuItem;
use crate::components::common::{ChannelContext, ServerContext, UiModalContext, UserProfileContext};
use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId, UserId},
};

#[allow(dead_code)]
pub(crate) fn build_message_menu(
    msg_el: &Element,
    clicked_el: &Element,
    selected_text: &str,
    modal_ctx: Option<&UiModalContext>,
    channel_ctx: Option<&ChannelContext>,
    server_ctx: Option<&ServerContext>,
    user_ctx: Option<&UserProfileContext>,
) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();

    let msg_id_str = msg_el.get_attribute("data-message-id").unwrap_or_default();
    let msg_content = msg_el
        .get_attribute("data-message-content")
        .unwrap_or_default();
    let author_name = msg_el.get_attribute("data-author-name").unwrap_or_default();
    let author_id_str = msg_el.get_attribute("data-author-id").unwrap_or_default();
    let channel_id_str = msg_el.get_attribute("data-channel-id").unwrap_or_default();

    let parsed_msg_id = MessageId::try_from(msg_id_str.as_str()).ok();
    let parsed_author_id = UserId::try_from(author_id_str.as_str()).ok();
    let parsed_channel_id = ChannelId::try_from(channel_id_str.as_str()).ok();

    let is_author = if let (Some(u_ctx), Some(author_id)) = (user_ctx, parsed_author_id) {
        u_ctx.user_id.with(|uid| uid.as_ref() == Some(&author_id))
            || u_ctx
                .username
                .with(|uname| !uname.is_empty() && uname == &author_name)
    } else if let Some(u_ctx) = user_ctx {
        u_ctx
            .username
            .with(|uname| !uname.is_empty() && uname == &author_name)
    } else {
        false
    };

    let found_message = channel_ctx.and_then(|c_ctx| {
        parsed_msg_id.and_then(|m_id| {
            c_ctx
                .messages
                .with(|msgs| msgs.iter().find(|m| m.id == m_id).cloned())
        })
    });

    let target_msg_fallback = MessageView {
        id: parsed_msg_id.unwrap_or_default(),
        channel_id: parsed_channel_id.unwrap_or_default(),
        author_id: parsed_author_id,
        author_name: author_name.clone(),
        content: msg_content.clone(),
        edited: false,
        reply_to_id: None,
        forwarded_from: None,
        created_at: time::OffsetDateTime::now_utc(),
        is_pending: false,
        is_failed: false,
    };

    // 1) Ответить (Reply)
    if let Some(m_ctx) = modal_ctx {
        let replying_sig = m_ctx.replying_to;
        let editing_sig = m_ctx.editing_message;
        let target_msg = found_message.clone().unwrap_or(target_msg_fallback.clone());
        items.push(ContextMenuItem {
            label: "Ответить".to_string(),
            icon: "reply",
            shortcut: None,
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || {
                editing_sig.set(None);
                replying_sig.set(Some(target_msg.clone()));
                #[cfg(target_arch = "wasm32")]
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        if let Ok(Some(editor)) = doc.query_selector(".chat-input") {
                            if let Ok(html_el) = editor.dyn_into::<web_sys::HtmlElement>() {
                                crate::components::common::focus_editor_at_end(&html_el);
                            }
                        }
                    }
                }
            }),
        });
    }

    // 2) Изменить (Edit) - only if author
    if is_author
        && let Some(m_ctx) = modal_ctx
    {
        let editing_sig = m_ctx.editing_message;
        let replying_sig = m_ctx.replying_to;
        let draft_sig = m_ctx.draft_message;
        let pending_att_sig = m_ctx.pending_attachments;
        let edit_content = msg_content.clone();
        let target_msg = found_message.clone().unwrap_or(target_msg_fallback.clone());
        items.push(ContextMenuItem {
            label: "Изменить".to_string(),
            icon: "edit",
            shortcut: Some("E"),
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || {
                replying_sig.set(None);
                editing_sig.set(Some(target_msg.clone()));

                let (clean_text, parsed_attachments) =
                    crate::components::common::media::extract_attachments_from_markdown(
                        &edit_content,
                    );

                draft_sig.set(clean_text.clone());
                pending_att_sig.set(parsed_attachments);

                    #[cfg(target_arch = "wasm32")]
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Ok(Some(editor)) = doc.query_selector(".chat-input") {
                                if let Ok(html_el) = editor.dyn_into::<web_sys::HtmlElement>() {
                                    crate::components::common::sync_editor_content(
                                        &html_el,
                                        &clean_text,
                                    );
                                    if clean_text.is_empty() {
                                        let _ = html_el.set_attribute("data-empty", "true");
                                    } else {
                                        let _ = html_el.remove_attribute("data-empty");
                                    }
                                    crate::components::common::focus_editor_at_end(&html_el);
                                }
                            }
                        }
                    }
                }),
            });
        }

    // 3) Переслать (Forward)
    if let Some(m_ctx) = modal_ctx {
        let fwd_msg_sig = m_ctx.forwarding_message;
        let fwd_open_sig = m_ctx.is_forward_modal_open;
        let target_msg = found_message.clone().unwrap_or(target_msg_fallback.clone());
        items.push(ContextMenuItem {
            label: "Переслать".to_string(),
            icon: "forward",
            shortcut: None,
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || {
                fwd_msg_sig.set(Some(target_msg.clone()));
                fwd_open_sig.set(true);
            }),
        });
    }

    // 4) Добавить в друзья / Написать сообщение - if another user's message
    if !is_author
        && let Some(target_uid) = parsed_author_id
    {
        let is_friend = user_ctx.is_some_and(|u| {
            u.friends.with(|list| {
                list.iter().any(|f| {
                    f.id == target_uid
                        || (!author_name.is_empty()
                            && f.username.eq_ignore_ascii_case(&author_name))
                })
            })
        });

            if !is_friend {
                #[allow(unused_variables)]
                let m_ctx_clone = modal_ctx.cloned();
                #[allow(unused_variables)]
                let a_name_clone = author_name.clone();
                items.push(ContextMenuItem {
                    label: "Добавить в друзья".to_string(),
                    icon: "user_plus",
                    shortcut: None,
                    danger: false,
                    separator_above: !items.is_empty(),
                    on_click: Rc::new(move || {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let m_ctx = m_ctx_clone.clone();
                            let target_display = if !a_name_clone.is_empty() {
                                format!("@{a_name_clone}")
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
            } else if let Some(m_ctx) = modal_ctx {
                let dm_name_sig = m_ctx.selected_dm_name;
                let home_tab_sig = m_ctx.home_tab;
                let s_ctx = server_ctx.cloned();
                let name_for_dm = author_name.clone();
                items.push(ContextMenuItem {
                    label: "Написать сообщение".to_string(),
                    icon: "message",
                    shortcut: None,
                    danger: false,
                    separator_above: !items.is_empty(),
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
        }

    // 5) Сохранить в загрузки (если кликнули по файлу или в сообщении есть вложение)
    let target_file = if let Ok(Some(file_el)) = clicked_el.closest(".chat-message-file") {
        let f_name = file_el.get_attribute("data-filename").unwrap_or_default();
        let f_id = file_el.get_attribute("data-file-id").unwrap_or_default();
        let f_url = file_el.get_attribute("data-file-url").unwrap_or_default();
        if !f_name.is_empty() {
            Some((f_name, f_id, f_url))
        } else {
            None
        }
    } else {
        let (_, atts) = crate::components::common::media::extract_message_media_and_text(&msg_content);
        atts.into_iter().next().map(|a| match a {
            crate::components::common::media::ParsedAttachment::File { name, file_id, url } => {
                (name, file_id.map(|id| id.to_string()).unwrap_or_default(), url)
            }
            crate::components::common::media::ParsedAttachment::Video { name, url } => {
                (name, String::new(), url)
            }
            crate::components::common::media::ParsedAttachment::Image { name, url } => {
                (name, String::new(), url)
            }
        })
    };

    #[allow(unused_variables)]
    if let Some((f_name, f_id, f_url)) = target_file {
        #[allow(unused_variables)]
        let m_ctx_opt = modal_ctx.cloned();
        items.push(ContextMenuItem {
            label: "Сохранить в загрузки".to_string(),
            icon: "download",
            shortcut: None,
            danger: false,
            separator_above: !items.is_empty(),
            on_click: Rc::new(move || {
                #[cfg(target_arch = "wasm32")]
                {
                    let m_ctx = m_ctx_opt.clone();
                    let fid = f_id.clone();
                    let fname = f_name.clone();
                    let furl = f_url.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::components::common::media::save_file_to_downloads_async(&fid, &fname, &furl).await {
                            Ok(_) => {
                                if let Some(m) = m_ctx.as_ref() {
                                    show_toast(m, false, format!("Файл «{fname}» сохранён в Загрузки"));
                                }
                            }
                            Err(err) => {
                                if let Some(m) = m_ctx.as_ref() {
                                    show_toast(m, true, format!("Не удалось сохранить файл: {err}"));
                                }
                            }
                        }
                    });
                }
            }),
        });
    }

    // 6) Копировать текст
    let text_to_copy = if !selected_text.is_empty() {
        selected_text.to_string()
    } else {
        msg_content.clone()
    };

    let t_copy = text_to_copy.clone();
    items.push(ContextMenuItem {
        label: "Копировать текст".to_string(),
        icon: "copy",
        shortcut: Some("Ctrl+C"),
        danger: false,
        separator_above: !items.is_empty(),
        on_click: Rc::new(move || copy_to_clipboard(&t_copy)),
    });

    // 6) Копировать ID сообщения
    let id_copy = msg_id_str.clone();
    items.push(ContextMenuItem {
        label: "Копировать ID сообщения".to_string(),
        icon: "info",
        shortcut: None,
        danger: false,
        separator_above: false,
        on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
    });

    // 7) Копировать ID автора
    if let Some(a_id) = parsed_author_id {
        let a_id_str = a_id.to_string();
        items.push(ContextMenuItem {
            label: "Копировать ID автора".to_string(),
            icon: "info",
            shortcut: None,
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || copy_to_clipboard(&a_id_str)),
        });
    }

    // 8) Удалить (Delete) - only if author
    if is_author
        && let (Some(m_id), Some(_ch_id)) = (parsed_msg_id, parsed_channel_id)
    {
        let c_ctx_opt = channel_ctx.copied();
        items.push(ContextMenuItem {
            label: "Удалить".to_string(),
            icon: "trash",
            shortcut: None,
            danger: true,
            separator_above: !items.is_empty(),
            on_click: Rc::new(move || {
                if let Some(c_ctx) = c_ctx_opt {
                    c_ctx.messages.update(|list| list.retain(|m| m.id != m_id));
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let ch_id = _ch_id;
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ =
                            crate::gateway::service::WsService::message_delete(ch_id, m_id).await;
                    });
                }
            }),
        });
    }

    items
}
