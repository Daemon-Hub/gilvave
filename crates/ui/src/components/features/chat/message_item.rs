use crate::{
    components::common::{MessageSegment, parse_message_segments},
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

    let (header_node, chained_node) = if is_first_in_group {
        (
            view! {
                div(class="message-header") {
                    span(class="author") { (author_name) }
                    span(class="timestamp") { (formatted_timestamp) }
                }
            },
            view! {},
        )
    } else {
        (
            view! {},
            view! {
                span(class="chained-time") { (formatted_timestamp) }
            },
        )
    };

    let full_content = message_view.content;
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

    view! {
        div(
            class=class_list,
            data-message-id=message_view.id.to_string(),
            data-message-content=full_content_attr,
            data-author-name=author_name_attr,
        ) {
            (avatar_node)
            div(class="message-body") {
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
