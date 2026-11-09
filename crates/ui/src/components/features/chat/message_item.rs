use crate::{
    components::{
        common::{ChannelContext, UiModalContext, UserProfileContext},
        ui::markdown::render_chat_markdown,
    },
    utils::to_local_datetime,
};
use gilvave_core::{dto::messages::MessageView, validation::MESSAGE_COLLAPSE_CHARS};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

fn visible_text_len(s: &str) -> usize {
    let mut count = 0;
    let mut in_url = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == ']' && chars.peek() == Some(&'(') {
            chars.next();
            in_url = true;
            continue;
        }
        if in_url {
            if c == ')' {
                in_url = false;
            }
            continue;
        }
        count += 1;
    }
    count
}

const TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[hour]:[minute]");

#[component(inline_props)]
pub fn MessageItem(
    message_view: MessageView,
    avatar_url: String,
    is_first_in_group: bool,
    is_last_in_group: bool,
) -> View {
    let local_dt = to_local_datetime(message_view.created_at);
    let formatted_timestamp = local_dt.format(TIME_FORMAT).unwrap_or_default();

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
    if message_view.is_failed {
        class_list.push_str(" is-failed");
    }

    let user_profile = use_context::<UserProfileContext>();
    let modal_context = use_context::<UiModalContext>();
    let is_own_message = user_profile
        .username
        .with(|u| !u.is_empty() && u == &message_view.author_name)
        || (message_view.author_id.is_some()
            && message_view.author_id == user_profile.user_id.get());

    let render_status = move || {
        if message_view.is_failed {
            view! {
                span(class="message-status is-failed", title="Не удалось отправить") {
                    svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2.5", stroke-linecap="round", stroke-linejoin="round") {
                        circle(cx="12", cy="12", r="10")
                        line(x1="12", y1="8", x2="12", y2="12")
                        line(x1="12", y1="16", x2="12.01", y2="16")
                    }
                    span(class="message-failed-label") { "Не удалось отправить" }
                }
            }
        } else if message_view.is_pending {
            view! {
                span(class="message-status is-sending", title="Отправляется...") {
                    svg(class="spin-loader-icon", viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2.5", stroke-linecap="round", stroke-linejoin="round") {
                        line(x1="12", y1="2", x2="12", y2="6")
                        line(x1="12", y1="18", x2="12", y2="22")
                        line(x1="4.93", y1="4.93", x2="7.76", y2="7.76")
                        line(x1="16.24", y1="16.24", x2="19.07", y2="19.07")
                        line(x1="2", y1="12", x2="6", y2="12")
                        line(x1="18", y1="12", x2="22", y2="12")
                        line(x1="4.93", y1="19.07", x2="7.76", y2="16.24")
                        line(x1="16.24", y1="7.76", x2="19.07", y2="4.93")
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

    let author_name = message_view.author_name.clone();
    let author_name_attr = author_name.clone();
    let author_id_attr = message_view
        .author_id
        .map(|id| id.to_string())
        .unwrap_or_default();

    let avatar_node = if is_first_in_group {
        let a_id = author_id_attr.clone();
        let a_name = author_name_attr.clone();
        if !avatar_url.is_empty() {
            view! {
                div(class="message-avatar", data-user-id=a_id, data-username=a_name) {
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
                div(class="message-avatar", data-user-id=a_id, data-username=a_name) {
                    span { (first_char) }
                }
            }
        }
    } else {
        view! {
            div(class="message-avatar-spacer")
        }
    };

    let edited_badge = if message_view.edited {
        view! {
            span(class="message-edited-badge") { "(ред.)" }
        }
    } else {
        view! {}
    };

    let (header_node, chained_node) = if is_first_in_group {
        let h_id = author_id_attr.clone();
        let h_name = author_name_attr.clone();
        (
            view! {
                div(class="message-header", data-user-id=h_id, data-username=h_name) {
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
    let is_long = visible_text_len(&full_content) > MESSAGE_COLLAPSE_CHARS;
    let truncated_content = if is_long {
        let prefix: String = full_content.chars().take(MESSAGE_COLLAPSE_CHARS).collect();
        format!("{prefix}…")
    } else {
        String::new()
    };

    let is_expanded = create_signal(false);
    let displayed_text = create_memo(move || {
        if !is_long || is_expanded.get() {
            full_content.clone()
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

    let on_bubble_click = move |e: web_sys::MouseEvent| {
        if let Some(target) = e.target()
            && let Ok(el) = target.dyn_into::<web_sys::HtmlElement>()
            && el.class_list().contains("chat-message-image")
            && let Ok(img) = el.dyn_into::<web_sys::HtmlImageElement>()
        {
            e.stop_propagation();
            modal_context
                .lightbox_image
                .set(Some((img.src(), img.alt())));
        }
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
                div(class="message-bubble", on:click=on_bubble_click) {
                    (move || {
                        let default_pack = modal_context.emoji_pack.get_untracked();
                        let (clean_text, attachments) = displayed_text.with(|text| {
                            crate::components::common::media::extract_message_media_and_text(text)
                        });

                        let text_node = if !clean_text.is_empty() {
                            let html_str = render_chat_markdown(&clean_text, default_pack);
                            let el = sycamore::web::tags::div()
                                .class("message-markdown-content")
                                .dangerously_set_inner_html(std::borrow::Cow::Owned(html_str));
                            View::from(el)
                        } else {
                            view! {}
                        };

                        let attachments_node = if !attachments.is_empty() {
                            view! {
                                super::attachments::MessageAttachmentsView(
                                    attachments=attachments,
                                    modal_ctx=modal_context,
                                    is_pending=message_view.is_pending,
                                    is_failed=message_view.is_failed,
                                )
                            }
                        } else {
                            view! {}
                        };

                        view! {
                            (text_node)
                            (attachments_node)
                        }
                    })
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
