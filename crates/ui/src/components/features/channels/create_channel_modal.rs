use gilvave_core::dto::channel::{ChannelCreateInfo, ChannelType};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::{
        common::{ChannelContext, ServerContext, UiModalContext},
        ui::icons::{CloseSmallIcon, HashIcon, Volume2Icon},
    },
    http::api::Api,
};

#[component]
pub fn CreateChannelModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();

    let channel_name = create_signal(String::new());

    let close = move |_| {
        modal_context.is_create_channel_open.set(false);
    };

    let handle_create = move |_| {
        let trimmed: String = channel_name.with(|raw| raw.trim().to_lowercase().replace(' ', "-"));
        if trimmed.is_empty() {
            return;
        }

        let Some(server_id) = server_context
            .current
            .with_untracked(|s| s.as_ref().map(|srv| srv.id))
        else {
            return;
        };

        spawn_local_scoped(async move {
            let info = ChannelCreateInfo {
                name: trimmed,
                r#type: modal_context.create_channel_type.get(),
            };

            if let Ok(new_channel) = Api::create_channel(server_id, info).await {
                match new_channel.r#type {
                    ChannelType::TEXT => {
                        channel_context
                            .text
                            .update(|list| list.push(new_channel.clone()));
                    }
                    ChannelType::VOICE => {
                        channel_context.voice.update(|list| list.push(new_channel));
                    }
                }
            }
            modal_context.is_create_channel_open.set(false);
        });
    };

    let is_text_type =
        create_memo(move || modal_context.create_channel_type.get() == ChannelType::TEXT);
    let is_voice_type =
        create_memo(move || modal_context.create_channel_type.get() == ChannelType::VOICE);
    
    let text_option_class = move || {
        if is_text_type.get() {
            "type-option active"
        } else {
            "type-option"
        }
    };

    let voice_option_class = move || {
        if is_voice_type.get() {
            "type-option active"
        } else {
            "type-option"
        }
    };

    view! {
        div(
            class="server-modal-overlay",
            on:click=close,
        ) {
            div(
                class="server-modal create-channel-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="server-modal-header") {
                    span { "Создать канал" }
                    button(class="modal-close-icon-btn", on:click=close, title="Закрыть") { CloseSmallIcon() }
                }

                div(class="create-channel-body") {
                    div(class="channel-type-selector") {
                        label(class="type-label") { "ТИП КАНАЛА" }
                        div(
                            class=text_option_class,
                            on:click=move |_| modal_context.create_channel_type.set(ChannelType::TEXT),
                        ) {
                            span(class="type-icon") { HashIcon() }
                            div(class="type-info") {
                                span(class="type-title") { "Текстовый" }
                                span(class="type-desc") { "Публикуйте сообщения, изображения, ссылки и мемы" }
                            }
                        }

                        div(
                            class=voice_option_class,
                            on:click=move |_| modal_context.create_channel_type.set(ChannelType::VOICE),
                        ) {
                            span(class="type-icon") { Volume2Icon() }
                            div(class="type-info") {
                                span(class="type-title") { "Голосовой" }
                                span(class="type-desc") { "Общайтесь голосом, видео и демонстрируйте экран" }
                            }
                        }
                    }

                    div(class="input-group") {
                        label { "НАЗВАНИЕ КАНАЛА" }
                        div(class="channel-name-input-wrapper") {
                            span(class="channel-prefix") {
                                (if is_text_type.get() {
                                    view! { HashIcon() }
                                } else {
                                    view! { Volume2Icon() }
                                })
                            }
                            input(
                                r#type="text",
                                placeholder="новый-канал",
                                bind:value=channel_name,
                            )
                        }
                    }

                    div(class="create-form-actions modal-footer-buttons") {
                        button(class="back-btn", on:click=close) { "Отмена" }
                        button(
                            class="submit-btn create-submit",
                            on:click=handle_create,
                        ) { "Создать канал" }
                    }
                }
            }
        }
    }
}
