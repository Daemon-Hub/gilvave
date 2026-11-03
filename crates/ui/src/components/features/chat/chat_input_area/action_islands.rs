//! Action status bars displayed above the chat input (replying and editing banners).

use sycamore::prelude::*;
use crate::components::{
    common::UiModalContext,
    ui::icons::{CloseSmallIcon, EditIcon},
};

#[component]
pub(crate) fn ActionIslands() -> View {
    let modal_context = use_context::<UiModalContext>();

    view! {
        (move || {
            if let Some(reply_msg) = modal_context.replying_to.get_clone() {
                let author = reply_msg.author_name;
                let snippet = if reply_msg.content.chars().count() > 60 {
                    let s: String = reply_msg.content.chars().take(60).collect();
                    format!("{s}…")
                } else {
                    reply_msg.content
                };
                view! {
                    div(class="chat-action-banner chat-reply-banner") {
                        div(class="banner-spine")
                        div(class="banner-info") {
                            span(class="banner-title") { (format!("Ответ пользователю @{author}")) }
                            span(class="banner-snippet") { (snippet) }
                        }
                        button(
                            r#type="button",
                            class="banner-close-btn",
                            title="Отменить ответ (Esc)",
                            on:click=move |_| modal_context.replying_to.set(None),
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else if modal_context.editing_message.with(|e| e.is_some()) {
                view! {
                    div(class="chat-edit-island-bar") {
                        div(class="chat-edit-island") {
                            div(class="island-edit-icon") {
                                EditIcon()
                            }
                            span(class="island-title") { "Редактирование" }
                            button(
                                r#type="button",
                                class="island-close-btn",
                                title="Отменить редактирование",
                                on:click=move |_| {
                                    modal_context.editing_message.set(None);
                                    modal_context.draft_message.set(String::new());
                                },
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                }
            } else {
                view! {}
            }
        })
    }
}
