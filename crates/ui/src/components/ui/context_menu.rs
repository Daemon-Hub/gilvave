use std::rc::Rc;
use sycamore::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::components::common::{ChannelContext, ServerContext, UiModalContext, UserProfileContext};
#[cfg(target_arch = "wasm32")]
use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId, UserId},
};

#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub icon: &'static str,
    pub shortcut: Option<&'static str>,
    pub danger: bool,
    pub separator_above: bool,
    #[allow(dead_code)]
    pub on_click: Rc<dyn Fn()>,
}

#[derive(Clone, Copy)]
pub struct ContextMenuContext {
    pub is_open: Signal<bool>,
    pub x: Signal<i32>,
    pub y: Signal<i32>,
    pub items: Signal<Vec<ContextMenuItem>>,
}

impl ContextMenuContext {
    pub fn new() -> Self {
        Self {
            is_open: create_signal(false),
            x: create_signal(0),
            y: create_signal(0),
            items: create_signal(Vec::new()),
        }
    }

    pub fn show(&self, x: i32, y: i32, items: Vec<ContextMenuItem>) {
        if items.is_empty() {
            return;
        }
        self.items.set(items);
        self.x.set(x);
        self.y.set(y);
        self.is_open.set(true);
    }

    pub fn close(&self) {
        self.is_open.set(false);
    }
}

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

pub fn handle_global_context_menu(
    event: &web_sys::MouseEvent,
    menu_ctx: &ContextMenuContext,
    modal_ctx: Option<&UiModalContext>,
    #[allow(unused_variables)] channel_ctx: Option<&ChannelContext>,
    _server_ctx: Option<&ServerContext>,
    #[allow(unused_variables)] user_ctx: Option<&UserProfileContext>,
) {
    event.prevent_default();
    let x = event.client_x();
    let y = event.client_y();

    let mut items = Vec::new();
    let selected_text = get_window_selection_text();

    #[cfg(target_arch = "wasm32")]
    if let Some(target) = event.target() {
        if let Ok(el) = target.dyn_into::<web_sys::Element>() {
            // 1. Check if clicked inside a message item
            if let Ok(Some(msg_el)) = el.closest("[data-message-id]") {
                let msg_id_str = msg_el.get_attribute("data-message-id").unwrap_or_default();
                let msg_content = msg_el
                    .get_attribute("data-message-content")
                    .unwrap_or_default();
                let author_name = msg_el
                    .get_attribute("data-author-name")
                    .unwrap_or_default();
                let author_id_str = msg_el
                    .get_attribute("data-author-id")
                    .unwrap_or_default();
                let channel_id_str = msg_el
                    .get_attribute("data-channel-id")
                    .unwrap_or_default();

                let parsed_msg_id = MessageId::try_from(msg_id_str.as_str()).ok();
                let parsed_author_id = UserId::try_from(author_id_str.as_str()).ok();
                let parsed_channel_id = ChannelId::try_from(channel_id_str.as_str()).ok();

                let is_author = if let (Some(u_ctx), Some(author_id)) = (user_ctx, parsed_author_id) {
                    u_ctx.user_id.with(|uid| uid.as_ref() == Some(&author_id))
                        || u_ctx.username.with(|uname| !uname.is_empty() && uname == &author_name)
                } else if let Some(u_ctx) = user_ctx {
                    u_ctx.username.with(|uname| !uname.is_empty() && uname == &author_name)
                } else {
                    false
                };

                let found_message = channel_ctx.and_then(|c_ctx| {
                    parsed_msg_id.and_then(|m_id| {
                        c_ctx.messages.with(|msgs| msgs.iter().find(|m| m.id == m_id).cloned())
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
                if is_author {
                    if let Some(m_ctx) = modal_ctx {
                        let editing_sig = m_ctx.editing_message;
                        let replying_sig = m_ctx.replying_to;
                        let draft_sig = m_ctx.draft_message;
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
                                draft_sig.set(edit_content.clone());
                                #[cfg(target_arch = "wasm32")]
                                if let Some(window) = web_sys::window() {
                                    if let Some(doc) = window.document() {
                                        if let Ok(Some(editor)) = doc.query_selector(".chat-input") {
                                            if let Ok(html_el) = editor.dyn_into::<web_sys::HtmlElement>() {
                                                crate::components::common::sync_editor_content(&html_el, &edit_content);
                                                let _ = html_el.remove_attribute("data-empty");
                                                crate::components::common::focus_editor_at_end(&html_el);
                                            }
                                        }
                                    }
                                }
                            }),
                        });
                    }
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

                // 4) Копировать текст
                let text_to_copy = if !selected_text.is_empty() {
                    selected_text.clone()
                } else {
                    msg_content.clone()
                };

                let t_copy = text_to_copy.clone();
                items.push(ContextMenuItem {
                    label: "Копировать текст".to_string(),
                    icon: "copy",
                    shortcut: Some("Ctrl+C"),
                    danger: false,
                    separator_above: true,
                    on_click: Rc::new(move || copy_to_clipboard(&t_copy)),
                });

                // 5) Копировать ID сообщения
                let id_copy = msg_id_str.clone();
                items.push(ContextMenuItem {
                    label: "Копировать ID сообщения".to_string(),
                    icon: "info",
                    shortcut: None,
                    danger: false,
                    separator_above: false,
                    on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
                });

                // 7) Удалить (Delete) - only if author
                if is_author {
                    if let (Some(m_id), Some(ch_id)) = (parsed_msg_id, parsed_channel_id) {
                        let c_ctx_opt = channel_ctx.copied();
                        items.push(ContextMenuItem {
                            label: "Удалить".to_string(),
                            icon: "trash",
                            shortcut: None,
                            danger: true,
                            separator_above: true,
                            on_click: Rc::new(move || {
                                if let Some(c_ctx) = c_ctx_opt {
                                    c_ctx.messages.update(|list| list.retain(|m| m.id != m_id));
                                }
                                #[cfg(target_arch = "wasm32")]
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = crate::gateway::service::WsService::message_delete(ch_id, m_id).await;
                                });
                            }),
                        });
                    }
                }

                menu_ctx.show(x, y, items);
                return;
            }

            // 2. Check if clicked inside an editable input / contenteditable / textarea
            let is_editable = el.tag_name().eq_ignore_ascii_case("INPUT")
                || el.tag_name().eq_ignore_ascii_case("TEXTAREA")
                || el.get_attribute("contenteditable").as_deref() == Some("true")
                || el
                    .closest("[contenteditable='true']")
                    .ok()
                    .flatten()
                    .is_some();

            if is_editable && !selected_text.is_empty() {
                let sel = selected_text.clone();
                items.push(ContextMenuItem {
                    label: "Копировать".to_string(),
                    icon: "copy",
                    shortcut: Some("Ctrl+C"),
                    danger: false,
                    separator_above: false,
                    on_click: Rc::new(move || copy_to_clipboard(&sel)),
                });

                menu_ctx.show(x, y, items);
                return;
            }

            // 3. Check if clicked on a channel item
            if let Ok(Some(ch_el)) = el.closest("[data-channel-id]") {
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

                let id_copy = ch_id.clone();
                items.push(ContextMenuItem {
                    label: "Скопировать ID канала".to_string(),
                    icon: "info",
                    shortcut: None,
                    danger: false,
                    separator_above: false,
                    on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
                });

                menu_ctx.show(x, y, items);
                return;
            }

            // 4. Check if clicked on a server icon
            if let Ok(Some(s_el)) = el.closest("[data-server-id]") {
                let s_id = s_el.get_attribute("data-server-id").unwrap_or_default();

                if let Some(m_ctx) = modal_ctx {
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

                if let Ok(s_id_typed) = gilvave_core::ids::ServerId::try_from(s_id.as_str()) {
                    let is_owned = crate::components::features::servers::server_overview_modal::get_owned_server_ids().contains(&s_id_typed);
                    items.push(ContextMenuItem {
                        label: if is_owned {
                            "Переместить в Сохранённые".to_string()
                        } else {
                            "Сделать моим сервером".to_string()
                        },
                        icon: if is_owned { "channel" } else { "user" },
                        shortcut: None,
                        danger: false,
                        separator_above: false,
                        on_click: Rc::new(move || {
                            crate::components::features::servers::server_overview_modal::toggle_owned_server_id(s_id_typed);
                        }),
                    });
                }

                let id_copy = s_id.clone();
                items.push(ContextMenuItem {
                    label: "Скопировать ID сервера".to_string(),
                    icon: "info",
                    shortcut: None,
                    danger: false,
                    separator_above: false,
                    on_click: Rc::new(move || copy_to_clipboard(&id_copy)),
                });

                menu_ctx.show(x, y, items);
                return;
            }
        }
    }

    // 5. Active text selection anywhere
    if !selected_text.is_empty() {
        let sel = selected_text.clone();
        items.push(ContextMenuItem {
            label: "Копировать выделенное".to_string(),
            icon: "copy",
            shortcut: Some("Ctrl+C"),
            danger: false,
            separator_above: false,
            on_click: Rc::new(move || copy_to_clipboard(&sel)),
        });
        menu_ctx.show(x, y, items);
        return;
    }

    // 6. Default general application menu
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

#[component(inline_props)]
fn MenuIcon(name: &'static str) -> View {
    match name {
        "reply" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="9 17 4 12 9 7")
                path(d="M20 18v-2a4 4 0 0 0-4-4H4")
            }
        },
        "forward" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="15 17 20 12 15 7")
                path(d="M4 18v-2a4 4 0 0 1 4-4h12")
            }
        },
        "quote" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z")
                path(d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z")
            }
        },
        "devtools" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="16 18 22 12 16 6")
                polyline(points="8 6 2 12 8 18")
            }
        },
        "copy" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                rect(x="9", y="9", width="13", height="13", rx="2", ry="2")
                path(d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1")
            }
        },
        "paste" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2")
                rect(x="8", y="2", width="8", height="4", rx="1", ry="1")
            }
        },
        "cut" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="6", cy="6", r="3")
                circle(cx="6", cy="18", r="3")
                line(x1="20", y1="4", x2="8.12", y2="15.88")
                line(x1="14.47", y1="14.48", x2="20", y2="20")
                line(x1="8.12", y1="8.12", x2="12", y2="12")
            }
        },
        "select_all" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                rect(x="3", y="3", width="18", height="18", rx="2", ry="2")
                polyline(points="9 11 12 14 22 4")
            }
        },
        "trash" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="3 6 5 6 21 6")
                path(d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2")
            }
        },
        "edit" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z")
            }
        },
        "settings" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="12", cy="12", r="3")
                path(d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z")
            }
        },
        "channel" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                line(x1="4", y1="9", x2="20", y2="9")
                line(x1="4", y1="15", x2="20", y2="15")
                line(x1="10", y1="3", x2="8", y2="21")
                line(x1="16", y1="3", x2="14", y2="21")
            }
        },
        "refresh" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M23 4v6h-6")
                path(d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10")
            }
        },
        "theme" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z")
            }
        },
        "info" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="12", cy="12", r="10")
                line(x1="12", y1="16", x2="12", y2="12")
                line(x1="12", y1="8", x2="12.01", y2="8")
            }
        },
        "upload" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
                polyline(points="17 8 12 3 7 8")
                line(x1="12", y1="3", x2="12", y2="15")
            }
        },
        _ => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="12", cy="12", r="1")
                circle(cx="19", cy="12", r="1")
                circle(cx="5", cy="12", r="1")
            }
        },
    }
}

#[component]
pub fn AppContextMenu() -> View {
    let ctx = use_context::<ContextMenuContext>();

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let ctx_clone = ctx;
            let on_key = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" && ctx_clone.is_open.get() {
                        ctx_clone.close();
                    }
                },
            );
            let _ =
                window.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
            on_key.forget();
        }
    }

    view! {
        (if ctx.is_open.get() {
            let raw_x = ctx.x.get();
            let raw_y = ctx.y.get();
            let items = ctx.items.get_clone();
            let item_count = items.len();
            let est_w = 224;
            let est_h = (item_count as i32 * 38) + 24;

            #[cfg(target_arch = "wasm32")]
            let (win_w, win_h) = if let Some(w) = web_sys::window() {
                let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1024.0) as i32;
                let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(768.0) as i32;
                (width, height)
            } else {
                (1024, 768)
            };
            #[cfg(not(target_arch = "wasm32"))]
            let (win_w, win_h) = (1024, 768);

            let opens_upwards = raw_y > (win_h / 2) || (raw_y + est_h > win_h - 16);
            let opens_leftwards = raw_x + est_w > win_w - 12;

            let (y_style, up_class) = if opens_upwards {
                let bottom_px = (win_h - raw_y).max(8);
                let max_h = (raw_y - 16).max(120);
                (format!("bottom: {bottom_px}px; max-height: {max_h}px;"), "open-up")
            } else {
                let top_px = raw_y.max(8);
                let max_h = (win_h - raw_y - 16).max(120);
                (format!("top: {top_px}px; max-height: {max_h}px;"), "open-down")
            };

            let (x_style, left_class) = if opens_leftwards {
                let right_px = (win_w - raw_x).max(8);
                (format!("right: {right_px}px;"), "open-left")
            } else {
                let left_px = raw_x.max(8);
                (format!("left: {left_px}px;"), "")
            };

            let menu_style = format!("{x_style} {y_style}");
            let menu_class = format!("app-context-menu {up_class} {left_class}").trim().to_string();

            let on_menu_click = move |e: web_sys::MouseEvent| {
                e.stop_propagation();
                #[cfg(target_arch = "wasm32")]
                if let Some(target) = e.target() {
                    if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                        if let Ok(Some(btn)) = el.closest("[data-item-index]") {
                            if let Ok(idx) = btn
                                .get_attribute("data-item-index")
                                .unwrap_or_default()
                                .parse::<usize>()
                            {
                                let action = ctx
                                    .items
                                    .with(|it| it.get(idx).map(|item| item.on_click.clone()));
                                ctx.close();
                                if let Some(action) = action {
                                    action();
                                }
                            }
                        }
                    }
                }
            };

            let item_views = items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| {
                    let mut btn_class = String::from("context-menu-item");
                    if item.danger {
                        btn_class.push_str(" is-danger");
                    }
                    let sep_view = if item.separator_above {
                        view! { div(class="context-menu-separator") }
                    } else {
                        view! {}
                    };
                    let shortcut_view = if let Some(sc) = item.shortcut {
                        view! { span(class="context-menu-shortcut") { (sc) } }
                    } else {
                        view! {}
                    };
                    view! {
                        (sep_view)
                        button(
                            r#type="button",
                            class=btn_class,
                            data-item-index=idx.to_string(),
                        ) {
                            span(class="context-menu-icon") {
                                MenuIcon(name=item.icon)
                            }
                            span(class="context-menu-label") { (item.label) }
                            (shortcut_view)
                        }
                    }
                })
                .collect::<Vec<_>>();

            view! {
                div(
                    class="context-menu-backdrop",
                    on:click=move |_| ctx.close(),
                    on:contextmenu=move |e: web_sys::MouseEvent| {
                        e.prevent_default();
                        ctx.close();
                    },
                )
                div(
                    class=menu_class,
                    style=menu_style,
                    role="menu",
                    tabindex="-1",
                    on:click=on_menu_click,
                    on:contextmenu=move |e: web_sys::MouseEvent| e.prevent_default(),
                ) {
                    (item_views)
                }
            }
        } else {
            view! {}
        })
    }
}
