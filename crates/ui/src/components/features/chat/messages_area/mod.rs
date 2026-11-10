//! Message feed container, reactive unread dividers, and gateway synchronization.

pub mod enriched;
pub mod feed;
pub mod header;

use futures_util::StreamExt;
use sycamore::{futures::spawn_local_scoped, prelude::*, web::queue_microtask};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement};

use self::enriched::compute_enriched_messages;
use self::feed::MessagesFeed;
use self::header::ChannelHeader;
use super::chat_input_area::send_channel_message;
use super::chat_scroll::{
    determine_channel_target_scroll,
    execute_initial_scroll_restoration, find_top_visible_message,
    get_channel_last_read, save_channel_scroll_state,
    ChannelScrollAnchor,
};
use crate::{
    cache::CacheService,
    components::common::{
        ChannelContext, ServerContext, UiModalContext, UserProfileContext,
    },
    gateway::{
        handler::{subscribe_gateway_events, GatewayEvent},
        service::WsService,
    },
};
use gilvave_core::ids::{ChannelId, MessageId};

// Re-export for seamless backwards compatibility
pub use super::chat_input_area::ChatInputArea;
pub use super::chat_scroll::save_channel_last_read;
pub use super::expanded_editor_modal::ExpandedMessageEditorModal;

#[component()]
pub fn MessagesArea() -> View {
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = use_context::<UserProfileContext>();
    let modal_context = use_context::<UiModalContext>();
    let channel_name = create_memo(move || {
        channel_context.current.with(|c| {
            c.as_ref()
                .map_or_else(|| "<UNKNOWN>".into(), |ch| ch.name.clone())
        })
    });

    let draft_message = modal_context.draft_message;
    let container = create_node_ref();

    let unread_marker_id = create_signal::<Option<MessageId>>(None);
    let initial_scroll_performed = create_signal::<bool>(false);
    let saved_read_marker = create_signal::<Option<(Option<MessageId>, Option<i64>)>>(None);
    let active_target_anchor = create_signal::<Option<ChannelScrollAnchor>>(None);
    let prev_channel_id = create_signal::<Option<ChannelId>>(None);
    let is_loading_history = create_signal::<bool>(false);
    let has_more_before = create_signal::<bool>(true);

    let on_scroll = move |event: Event| {
        let el: HtmlElement = match event.current_target() {
            Some(t) => t.unchecked_into(),
            None => return,
        };

        const SCROLL_THRESHOLD: i32 = 50;

        let top = el.scroll_top();
        let sh = el.scroll_height();
        let ch = el.client_height();

        let is_top = top <= 20;
        let is_bottom = (sh - ch - top).abs() <= SCROLL_THRESHOLD;

        let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)) else {
            return;
        };

        // Сохраняем состояние скролла только после выполнения начального позиционирования
        if initial_scroll_performed.get() {
            if is_bottom {
                // Пользователь в самом низу: обновляем прочитанные сообщения и фиксируем Bottom
                if let Some(last_msg) = channel_context.messages.with(|m| m.last().cloned()) {
                    save_channel_last_read(
                        ch_id,
                        last_msg.id,
                        last_msg.created_at.unix_timestamp(),
                    );
                }
                save_channel_scroll_state(ch_id, true, None, 0);
                active_target_anchor.set(Some(ChannelScrollAnchor::Bottom));
            } else {
                // Пользователь пролистал вверх: фиксируем конкретное видимое сообщение
                let (anchor_id, offset) = find_top_visible_message(&el);
                if let Some(a_id) = anchor_id {
                    save_channel_scroll_state(ch_id, false, Some(a_id), offset);
                    active_target_anchor.set(Some(ChannelScrollAnchor::Message {
                        message_id: a_id,
                        offset,
                    }));
                }
            }
        }

        if is_top && !is_loading_history.get() && has_more_before.get() {
            console_log!("fetching older channel history before...");
            is_loading_history.set(true);
            spawn_local_scoped(async move {
                let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id))
                else {
                    is_loading_history.set(false);
                    return;
                };
                let Some(first_dt) = channel_context
                    .messages
                    .with(|m| m.first().map(|msg| msg.created_at))
                else {
                    is_loading_history.set(false);
                    return;
                };
                let _ = WsService::channel_history_before(ch_id, first_dt).await;
            });
        }
        if is_bottom {
            console_log!("bottom message");
            spawn_local_scoped(async move {
                let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id))
                else {
                    return;
                };
                let Some(last_dt) = channel_context
                    .messages
                    .with(|m| m.last().map(|msg| msg.created_at))
                else {
                    return;
                };
                let _ = WsService::channel_history_after(ch_id, last_dt).await;
            });
        }
    };

    create_effect(move || {
        let cur = channel_context.current.get_clone();

        // 1. При переключении канала надежно сохраняем состояние предыдущего канала
        if let Some(prev_id) = prev_channel_id.get_untracked()
            && cur.as_ref().map(|c| c.id) != Some(prev_id)
            && let Some(node) = container.try_get()
        {
            let el: HtmlElement = node.unchecked_into();
            const SCROLL_THRESHOLD: i32 = 50;
            let is_bottom = (el.scroll_height() - el.client_height() - el.scroll_top()).abs() <= SCROLL_THRESHOLD;
            let fits_screen = el.scroll_height() <= el.client_height() + 20;

            if is_bottom || fits_screen {
                if let Some(last_msg) = channel_context.messages.with_untracked(|m| m.last().cloned()) {
                    save_channel_last_read(prev_id, last_msg.id, last_msg.created_at.unix_timestamp());
                }
                save_channel_scroll_state(prev_id, true, None, 0);
            } else {
                let (anchor_id, offset) = find_top_visible_message(&el);
                if let Some(a_id) = anchor_id {
                    save_channel_scroll_state(prev_id, false, Some(a_id), offset);
                }
            }
        }
        prev_channel_id.set(cur.as_ref().map(|c| c.id));

        channel_context.messages.set(vec![]);
        initial_scroll_performed.set(false);
        unread_marker_id.set(None);
        active_target_anchor.set(None);
        is_loading_history.set(false);
        has_more_before.set(true);

        let saved = cur.as_ref().and_then(|ch| get_channel_last_read(ch.id));
        saved_read_marker.set(saved);

        if let Some(ch) = cur {
            spawn_local_scoped(async move {
                let cached = CacheService::get_cached_messages(ch.id, 100, None).await;
                if !cached.is_empty() {
                    channel_context.messages.update(|list| {
                        if list.is_empty() {
                            *list = cached.clone();
                        } else {
                            for msg in &cached {
                                if !list.iter().any(|m| m.id == msg.id) {
                                    list.push(msg.clone());
                                }
                            }
                            list.sort_by_key(|m| m.created_at);
                        }
                    });

                    if !initial_scroll_performed.get() {
                        let (target_anchor, marker_id) = determine_channel_target_scroll(
                            ch.id,
                            saved,
                            &cached,
                        );

                        if saved.is_none() {
                            // Начальный базис для нового канала
                            if let Some(last_msg) = cached.last() {
                                save_channel_last_read(ch.id, last_msg.id, last_msg.created_at.unix_timestamp());
                            }
                            save_channel_scroll_state(ch.id, true, None, 0);
                            saved_read_marker.set(cached.last().map(|m| (Some(m.id), Some(m.created_at.unix_timestamp()))));
                        }

                        unread_marker_id.set(marker_id);
                        active_target_anchor.set(Some(target_anchor.clone()));

                        if let Some(node) = container.try_get() {
                            let el: HtmlElement = node.unchecked_into();
                            execute_initial_scroll_restoration(el, target_anchor);
                            initial_scroll_performed.set(true);
                        }
                    }
                }
            });
        }
    });

    spawn_local_scoped(async move {
        web_sys::console::log_1(&"[MESSAGES_AREA] subscribing to gateway events...".into());
        let mut events = subscribe_gateway_events();
        while let Some(event) = events.next().await {
            match event {
                GatewayEvent::MessageNew(msg) => {
                    web_sys::console::log_1(
                        &format!("[MESSAGES_AREA] received MessageNew: {:?}", msg).into(),
                    );
                    let cur_channel_id =
                        channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if cur_channel_id != Some(msg.channel_id) {
                        continue;
                    }

                    let Some(node) = container.try_get() else {
                        continue;
                    };
                    let el = node.unchecked_into::<HtmlElement>();
                    let old_scroll_top = el.scroll_top();
                    let old_scroll_height = el.scroll_height();
                    let old_client_height = el.client_height();

                    const SCROLL_THRESHOLD: i32 = 80;
                    let is_bottom = (old_scroll_height - old_client_height - old_scroll_top).abs()
                        <= SCROLL_THRESHOLD;
                    let is_own = user_profile
                        .username
                        .with(|u| !u.is_empty() && u == &msg.author_name)
                        || (msg.author_id.is_some() && msg.author_id == user_profile.user_id.get());

                    const MAX_LOADED_MESSAGES: usize = 300;
                    let msg_id = msg.id;
                    let msg_ch_id = msg.channel_id;
                    let msg_ts = msg.created_at.unix_timestamp();
                    let msg_to_cache = msg.clone();
                    spawn_local_scoped(async move {
                        CacheService::save_cached_message(msg_to_cache).await;
                    });

                    channel_context.messages.update(|list| {
                        // Prevent duplicate if already received
                        if list.iter().any(|m| m.id == msg_id) {
                            return;
                        }

                        // Remove matching pending optimistic message
                        if is_own {
                            if let Some(idx) = list.iter().position(|m| m.is_pending) {
                                list.remove(idx);
                            }
                        } else {
                            list.retain(|m| !m.is_pending);
                        }
                        list.push(msg);
                        if list.len() > MAX_LOADED_MESSAGES {
                            list.remove(0);
                        }
                        list.sort_by_key(|m| m.created_at);
                    });

                    queue_microtask(move || {
                        if is_own || is_bottom {
                            el.set_scroll_top(el.scroll_height());
                            if let Some(window) = web_sys::window() {
                                let el_clone = el.clone();
                                let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                                    el_clone.set_scroll_top(el_clone.scroll_height());
                                });
                                let _ = window.request_animation_frame(cb.unchecked_ref());
                            }
                            save_channel_last_read(msg_ch_id, msg_id, msg_ts);
                            save_channel_scroll_state(msg_ch_id, true, None, 0);
                        } else {
                            if unread_marker_id.get().is_none() {
                                unread_marker_id.set(Some(msg_id));
                            }
                            let delta = el.scroll_height() - old_scroll_height;
                            if delta > 0 {
                                el.set_scroll_top(old_scroll_top + delta);
                            }
                        }
                    });
                }
                GatewayEvent::ChannelHistoryBefore(messages) => {
                    web_sys::console::log_1(
                        &format!(
                            "[MESSAGES_AREA] received ChannelHistoryBefore with {} items",
                            messages.len()
                        )
                        .into(),
                    );
                    is_loading_history.set(false);
                    if messages.is_empty() {
                        has_more_before.set(false);
                    }
                    let Some(node) = container.try_get() else {
                        continue;
                    };
                    let el = node.unchecked_into::<HtmlElement>();
                    let old_scroll_top = el.scroll_top();
                    let old_scroll_height = el.scroll_height();
                    let was_empty = channel_context.messages.with(|m| m.is_empty());
                    let has_initial_scroll = initial_scroll_performed.get();

                    let cur_ch_id = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if let Some(ch_id) = cur_ch_id {
                        let msgs_clone = messages.clone();
                        spawn_local_scoped(async move {
                            CacheService::sync_cached_messages(ch_id, msgs_clone).await;
                        });
                    }

                    const MAX_LOADED_MESSAGES: usize = 300;
                    channel_context.messages.update(|list| {
                        list.retain(|m| !m.is_pending);

                        if !messages.is_empty() {
                            let min_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .min()
                                .expect("non-empty");
                            let max_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .max()
                                .expect("non-empty");
                            let server_set: std::collections::HashSet<_> =
                                messages.iter().map(|m| m.id).collect();

                            // Reconcile: delete messages absent from server in this time window
                            list.retain(|m| {
                                if m.created_at >= min_time && m.created_at <= max_time {
                                    server_set.contains(&m.id)
                                } else {
                                    true
                                }
                            });

                            // Reconcile: update edited messages and insert new ones
                            for msg in messages {
                                if let Some(existing) = list.iter_mut().find(|m| m.id == msg.id) {
                                    existing.content = msg.content;
                                    existing.edited = msg.edited;
                                    existing.forwarded_from = msg.forwarded_from;
                                    existing.reply_to_id = msg.reply_to_id;
                                } else {
                                    list.push(msg);
                                }
                            }
                        }

                        list.sort_by_key(|m| m.created_at);
                        if list.len() > MAX_LOADED_MESSAGES {
                            let excess = list.len() - MAX_LOADED_MESSAGES;
                            list.drain(0..excess);
                        }
                    });

                    let all_msgs = channel_context.messages.get_clone();
                    let current_ch_id = cur_ch_id.unwrap_or_default();

                    if was_empty || !has_initial_scroll {
                        let (target_anchor, marker_id) = determine_channel_target_scroll(
                            current_ch_id,
                            saved_read_marker.get(),
                            &all_msgs,
                        );

                        if saved_read_marker.get().is_none() && let Some(last_msg) = all_msgs.last() {
                            save_channel_last_read(current_ch_id, last_msg.id, last_msg.created_at.unix_timestamp());
                            save_channel_scroll_state(current_ch_id, true, None, 0);
                            saved_read_marker.set(Some((Some(last_msg.id), Some(last_msg.created_at.unix_timestamp()))));
                        }

                        unread_marker_id.set(marker_id);
                        active_target_anchor.set(Some(target_anchor.clone()));
                        execute_initial_scroll_restoration(el.clone(), target_anchor);
                        initial_scroll_performed.set(true);
                    } else {
                        let el_clone = el.clone();
                        queue_microtask(move || {
                            let delta = el_clone.scroll_height() - old_scroll_height;
                            if delta > 0 {
                                el_clone.set_scroll_top(old_scroll_top + delta);
                            }
                        });
                    }
                }
                GatewayEvent::ChannelHistoryAfter(messages) => {
                    web_sys::console::log_1(
                        &format!(
                            "[MESSAGES_AREA] received ChannelHistoryAfter with {} items",
                            messages.len()
                        )
                        .into(),
                    );
                    let cur_ch_id = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if let Some(ch_id) = cur_ch_id {
                        let msgs_clone = messages.clone();
                        spawn_local_scoped(async move {
                            CacheService::sync_cached_messages(ch_id, msgs_clone).await;
                        });
                    }

                    const MAX_LOADED_MESSAGES: usize = 300;
                    channel_context.messages.update(|list| {
                        list.retain(|m| !m.is_pending);

                        if !messages.is_empty() {
                            let min_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .min()
                                .expect("non-empty");
                            let max_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .max()
                                .expect("non-empty");
                            let server_set: std::collections::HashSet<_> =
                                messages.iter().map(|m| m.id).collect();

                            list.retain(|m| {
                                if m.created_at >= min_time && m.created_at <= max_time {
                                    server_set.contains(&m.id)
                                } else {
                                    true
                                }
                            });

                            for msg in messages {
                                if let Some(existing) = list.iter_mut().find(|m| m.id == msg.id) {
                                    existing.content = msg.content;
                                    existing.edited = msg.edited;
                                    existing.forwarded_from = msg.forwarded_from;
                                    existing.reply_to_id = msg.reply_to_id;
                                } else {
                                    list.push(msg);
                                }
                            }
                        }

                        list.sort_by_key(|m| m.created_at);
                        if list.len() > MAX_LOADED_MESSAGES {
                            let excess = list.len() - MAX_LOADED_MESSAGES;
                            list.drain(0..excess);
                        }
                    });
                }
                GatewayEvent::MessageUpdate(updated) => {
                    web_sys::console::log_1(
                        &format!("[MESSAGES_AREA] received MessageUpdate: {:?}", updated).into(),
                    );
                    let cur_channel_id =
                        channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if cur_channel_id != Some(updated.channel_id) {
                        continue;
                    }

                    let upd_id = updated.id;
                    let upd_content = updated.content.clone();
                    let upd_edited = updated.edited;
                    spawn_local_scoped(async move {
                        CacheService::update_cached_message(upd_id, upd_content, upd_edited).await;
                    });

                    channel_context.messages.update(|list| {
                        if let Some(target) = list.iter_mut().find(|m| m.id == updated.id) {
                            target.content = updated.content;
                            target.edited = true;
                        }
                    });
                }
                GatewayEvent::MessageDelete {
                    channel_id,
                    message_id,
                } => {
                    web_sys::console::log_1(
                        &format!("[MESSAGES_AREA] received MessageDelete: message_id={message_id}")
                            .into(),
                    );
                    let cur_channel_id =
                        channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if cur_channel_id != Some(channel_id) {
                        continue;
                    }

                    let del_id = message_id;
                    spawn_local_scoped(async move {
                        CacheService::delete_cached_message(del_id).await;
                    });

                    channel_context.messages.update(|list| {
                        list.retain(|m| m.id != message_id);
                    });
                    if modal_context
                        .replying_to
                        .with(|r| r.as_ref().map(|m| m.id) == Some(message_id))
                    {
                        modal_context.replying_to.set(None);
                    }
                    if modal_context
                        .editing_message
                        .with(|e| e.as_ref().map(|m| m.id) == Some(message_id))
                    {
                        modal_context.editing_message.set(None);
                        modal_context.draft_message.set(String::new());
                    }
                }
                _ => {}
            }
        }
    });

    let on_back = move |_| {
        if let Some(ch) = channel_context.current.get_clone()
            && let Some(node) = container.try_get()
        {
            let el: HtmlElement = node.unchecked_into();
            const SCROLL_THRESHOLD: i32 = 50;
            let is_bottom = (el.scroll_height() - el.client_height() - el.scroll_top()).abs() <= SCROLL_THRESHOLD;
            let fits_screen = el.scroll_height() <= el.client_height() + 20;

            if is_bottom || fits_screen {
                if let Some(last_msg) = channel_context.messages.with_untracked(|m| m.last().cloned()) {
                    save_channel_last_read(ch.id, last_msg.id, last_msg.created_at.unix_timestamp());
                }
                save_channel_scroll_state(ch.id, true, None, 0);
            } else {
                let (anchor_id, offset) = find_top_visible_message(&el);
                if let Some(a_id) = anchor_id {
                    save_channel_scroll_state(ch.id, false, Some(a_id), offset);
                }
            }
        }
        channel_context.current.set(None);
    };

    let enriched_messages = create_memo(move || {
        let cur_unread_marker = unread_marker_id.get();
        channel_context.messages.with(|msgs| {
            server_context.members.with(|members| {
                user_profile.username.with(|my_username| {
                    user_profile.avatar.with(|my_avatar| {
                        compute_enriched_messages(
                            msgs,
                            members,
                            my_username,
                            my_avatar,
                            cur_unread_marker,
                        )
                    })
                })
            })
        })
    });

    view! {
        div(class="messages-area") {
            ChannelHeader(channel_name=channel_name, on_back=on_back)

            MessagesFeed(
                channel_name=channel_name,
                enriched_messages=enriched_messages,
                container=container,
                on_scroll=on_scroll,
            )

            ChatInputArea(
                on_send=move || send_channel_message(
                    channel_context,
                    modal_context,
                    draft_message,
                    user_profile,
                    Some(container),
                ),
            )
        }
    }
}
