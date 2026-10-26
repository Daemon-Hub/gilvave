use crate::{
    components::common::{ChannelContext, MessageSegment, UserProfileContext, parse_message_segments},
    utils::to_local_datetime,
};
use gilvave_core::{dto::messages::MessageView, validation::MESSAGE_COLLAPSE_CHARS};
use sycamore::prelude::*;
use time::format_description::parse_borrowed;

#[component(inline_props)]
pub fn MessageItem(
    message_view: MessageView,
    avatar_url: String,
    is_first_in_group: bool,
    is_last_in_group: bool,
) -> View {
    let format = parse_borrowed::<3>("[hour]:[minute]").unwrap();
    let local_dt = to_local_datetime(message_view.created_at);
    let formatted_timestamp = local_dt.format(&format).unwrap();

    let mut class_list = String::from("message");
    if !is_first_in_group {
        class_list.push_str(" chained");
    }
    if is_first_in_group {
        class_list.push_str(" group-first");
    }
    if is_last_in_group {
        class_list.push_str(" group-last");
    }
    if message_view.is_pending {
        class_list.push_str(" is-pending");
    }

    let user_profile = use_context::<UserProfileContext>();
    let is_own_message = user_profile.username.with(|u| !u.is_empty() && u == &message_view.author_name)
        || (message_view.author_id.is_some() && message_view.author_id == user_profile.user_id.get());

    let render_status = move || {
        if message_view.is_pending {
            view! {
                span(class="message-status is-sending", title="Отправляется...") {
                    svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                        circle(cx="12", cy="12", r="10")
                        polyline(points="12 6 12 12 16 14")
                    }
                }
            }
        } else if is_own_message {
            view! {
                span(class="message-status is-delivered", title="Отправлено") {
                    svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2.5", stroke-linecap="round", stroke-linejoin="round") {
                        polyline(points="20 6 9 17 4 12")
                    }
                }
            }
        } else {
            view! {}
        }
    };

    let avatar_node = if is_first_in_group {
        if !avatar_url.is_empty() {
            view! {
                div(class="message-avatar") {
                    img(class="message-avatar-img", src=avatar_url, alt="")
                }
            }
        } else {
            let first_char = message_view
                .author_name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            view! {
                div(class="message-avatar") {
                    span { (first_char) }
                }
            }
        }
    } else {
        view! {
            div(class="message-avatar-spacer")
        }
    };

    let author_name = message_view.author_name.clone();
    let author_name_attr = author_name.clone();

    let edited_badge = if message_view.edited {
        view! {
            span(class="message-edited-badge") { "(ред.)" }
        }
    } else {
        view! {}
    };

    let (header_node, chained_node) = if is_first_in_group {
        (
            view! {
                div(class="message-header") {
                    span(class="author") { (author_name) }
                    span(class="timestamp") {
                        (formatted_timestamp)
                        (render_status())
                    }
                    (edited_badge)
                }
            },
            view! {},
        )
    } else {
        (
            view! {},
            view! {
                span(class="chained-time") {
                    (formatted_timestamp)
                    (render_status())
                    (if message_view.edited {
                        view! { span(class="chained-edited") { " (ред.)" } }
                    } else {
                        view! {}
                    })
                }
            },
        )
    };

    let full_content = message_view.content.clone();
    let full_content_attr = full_content.clone();
    let full_content_for_memo = full_content.clone();
    let char_count = full_content.chars().count();
    let is_long = char_count > MESSAGE_COLLAPSE_CHARS;
    let truncated_content = if is_long {
        let prefix: String = full_content.chars().take(MESSAGE_COLLAPSE_CHARS).collect();
        format!("{prefix}…")
    } else {
        full_content.clone()
    };

    let is_expanded = create_signal(false);
    let displayed_text = create_memo(move || {
        if !is_long || is_expanded.get() {
            full_content_for_memo.clone()
        } else {
            truncated_content.clone()
        }
    });

    let channel_context = use_context::<ChannelContext>();
    let reply_view = move || {
        if let Some(reply_id) = message_view.reply_to_id {
            let replied_msg = channel_context
                .messages
                .with(|msgs| msgs.iter().find(|m| m.id == reply_id).cloned());

            let on_reply_click = move |_| {
                #[cfg(target_arch = "wasm32")]
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        if let Ok(Some(el)) =
                            doc.query_selector(&format!("[data-message-id='{}']", reply_id))
                        {
                            el.scroll_into_view_with_bool(false);
                            let _ = el.set_attribute("data-highlight", "true");
                            let el_clone = el.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                gloo_timers::future::sleep(core::time::Duration::from_millis(1500))
                                    .await;
                                let _ = el_clone.remove_attribute("data-highlight");
                            });
                        }
                    }
                }
            };

            if let Some(r_msg) = replied_msg {
                let r_author = r_msg.author_name;
                let r_snippet = if r_msg.content.chars().count() > 50 {
                    let s: String = r_msg.content.chars().take(50).collect();
                    format!("{s}…")
                } else {
                    r_msg.content
                };
                view! {
                    div(class="message-reply-preview-bar", on:click=on_reply_click) {
                        div(class="reply-spine")
                        span(class="reply-author-tag") { (format!("@{r_author}")) }
                        span(class="reply-snippet") { (r_snippet) }
                    }
                }
            } else {
                view! {
                    div(class="message-reply-preview-bar is-deleted") {
                        div(class="reply-spine")
                        span(class="reply-snippet is-deleted") { "Сообщение удалено" }
                    }
                }
            }
        } else {
            view! {}
        }
    };

    let forward_node = if let Some(fwd) = message_view.parse_forwarded_from() {
        let fwd_label = if let Some(ch) = fwd.channel_name {
            format!("Переслано из #{ch}")
        } else if let Some(author) = fwd.author_name {
            format!("Переслано от {author}")
        } else {
            "Пересланное сообщение".to_string()
        };
        view! {
            div(class="message-forwarded-bar") {
                svg(
                    xmlns="http://www.w3.org/2000/svg",
                    viewBox="0 0 24 24",
                    fill="none",
                    stroke="currentColor",
                    stroke-width="2",
                    stroke-linecap="round",
                    stroke-linejoin="round",
                    class="forward-icon",
                ) {
                    polyline(points="15 17 20 12 15 7")
                    path(d="M4 18v-2a4 4 0 0 1 4-4h12")
                }
                span(class="forward-label") { (fwd_label) }
            }
        }
    } else {
        view! {}
    };

    view! {
        div(
            class=class_list,
            data-message-id=message_view.id.to_string(),
            data-message-content=full_content_attr,
            data-author-name=author_name_attr,
            data-author-id=message_view.author_id.map(|id| id.to_string()).unwrap_or_default(),
            data-channel-id=message_view.channel_id.to_string(),
        ) {
            (avatar_node)
            div(class="message-body") {
                (forward_node)
                (reply_view)
                (header_node)
                div(class="message-bubble") {
                    p {
                        (move || {
                            let text = displayed_text.get_clone();
                            let segments = parse_message_segments(&text);
                            let views: Vec<View> = segments
                                .into_iter()
                                .map(|seg| match seg {
                                    MessageSegment::Text(t) => view! { (t) },
                                    MessageSegment::Emoji { char, url, name } => view! {
                                        img(
                                            class="chat-inline-emoji",
                                            src=url,
                                            alt=char,
                                            title=name,
                                            draggable="false",
                                        )
                                    },
                                })
                                .collect();
                            views
                        })
                    }
                    (if is_long {
                        let toggle_label = move || {
                            if is_expanded.get() {
                                "Свернуть"
                            } else {
                                "Далее"
                            }
                        };
                        view! {
                            button(
                                class="message-read-more-btn",
                                r#type="button",
                                on:click=move |_| is_expanded.set(!is_expanded.get()),
                            ) {
                                (toggle_label)
                            }
                        }
                    } else {
                        view! {}
                    })
                    (chained_node)
                }
            }
        }
    }
}
