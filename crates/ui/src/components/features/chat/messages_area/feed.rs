//! Scrollable message list feed with keyed message views, date dividers, and unread dividers.

use sycamore::prelude::*;

use super::enriched::{EnrichedMessage, hash_str};
use super::header::WelcomeMessage;
use crate::components::features::chat::message_item::MessageItem;

#[component(inline_props)]
pub fn MessagesFeed<F>(
    channel_name: ReadSignal<String>,
    enriched_messages: ReadSignal<Vec<EnrichedMessage>>,
    container: NodeRef,
    on_scroll: F,
) -> View
where
    F: Fn(web_sys::Event) + 'static,
{
    view! {
        div(class="messages-list", r#ref=container, on:scroll=on_scroll) {
            WelcomeMessage(channel_name=channel_name)

            Keyed(
                list=enriched_messages,
                key=|em| (
                    em.message.id,
                    hash_str(&em.message.content),
                    em.message.edited,
                    em.message.is_pending,
                    em.message.is_failed,
                    em.message.reply_to_id,
                    em.is_first,
                    em.is_last,
                    em.date_divider.as_deref().map(hash_str),
                    em.is_unread_divider,
                    hash_str(&em.avatar_url),
                ),
                view=|em| {
                    let date_view = if let Some(date_str) = em.date_divider {
                        view! {
                            div(class="chat-date-divider") {
                                span(class="date-badge") { (date_str) }
                            }
                        }
                    } else {
                        view! {}
                    };

                    let unread_view = if em.is_unread_divider {
                        view! {
                            div(
                                class="chat-unread-divider",
                                data-unread-marker=em.message.id.to_string(),
                            ) {
                                span(class="unread-badge") { "Непрочитанные сообщения" }
                            }
                        }
                    } else {
                        view! {}
                    };

                    view! {
                        (date_view)
                        (unread_view)
                        MessageItem(
                            message_view=em.message,
                            avatar_url=em.avatar_url,
                            is_first_in_group=em.is_first,
                            is_last_in_group=em.is_last,
                        )
                    }
                },
            )
        }
    }
}
