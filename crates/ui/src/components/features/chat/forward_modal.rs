use sycamore::{futures::spawn_local_scoped, prelude::*};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::{
    components::{
        common::{ChannelContext, ServerContext, UiModalContext},
        ui::icons::{CloseSmallIcon, HashIcon},
    },
    http::api::Api,
};
use gilvave_core::{
    dto::{messages::ForwardedFrom, servers::channels::ChannelType},
    ids::{ChannelId, ServerId},
};

#[component]
pub fn ForwardMessageModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();

    let initial_srv = server_context
        .current
        .with_untracked(|s| s.as_ref().map(|srv| srv.id))
        .or_else(|| {
            server_context
                .list
                .with_untracked(|list| list.first().map(|s| s.id))
        });

    let selected_server_id = create_signal::<Option<ServerId>>(initial_srv);
    let channels_list =
        create_signal::<Vec<gilvave_core::dto::servers::channels::ChannelView>>(vec![]);
    let selected_channel_id = create_signal::<Option<ChannelId>>(None);
    let is_sending = create_signal(false);

    let close = move |_| {
        modal_context.is_forward_modal_open.set(false);
        modal_context.forwarding_message.set(None);
    };

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let on_key = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" && modal_context.is_forward_modal_open.get() {
                        modal_context.is_forward_modal_open.set(false);
                        modal_context.forwarding_message.set(None);
                    }
                },
            );
            let _ =
                window.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
            let window_clone = window.clone();
            on_cleanup(move || {
                let _ = window_clone.remove_event_listener_with_callback(
                    "keydown",
                    on_key.as_ref().unchecked_ref(),
                );
            });
        }
    }

    create_effect(move || {
        if let Some(srv_id) = selected_server_id.get() {
            let is_curr = server_context
                .current
                .with(|c| c.as_ref().map(|s| s.id) == Some(srv_id));
            if is_curr {
                let current_text = channel_context.text.get_clone();
                channels_list.set(current_text);
            } else {
                spawn_local_scoped(async move {
                    if let Ok(chs) = Api::get_server_channels(srv_id).await {
                        let text_chs: Vec<_> = chs
                            .into_iter()
                            .filter(|c| c.r#type == ChannelType::TEXT)
                            .collect();
                        channels_list.set(text_chs);
                    }
                });
            }
        } else {
            channels_list.set(vec![]);
        }
    });

    let handle_forward = move |_| {
        if is_sending.get() {
            return;
        }
        let Some(target_ch_id) = selected_channel_id.get() else {
            return;
        };
        let Some(msg) = modal_context.forwarding_message.get_clone() else {
            return;
        };

        is_sending.set(true);

        let curr_ch_name = channel_context
            .current
            .with(|c| c.as_ref().map(|ch| ch.name.clone()));
        let curr_srv_name = server_context
            .current
            .with(|s| s.as_ref().map(|srv| srv.name.clone()));

        let fwd_info = ForwardedFrom {
            author_name: Some(msg.author_name.clone()),
            server_name: curr_srv_name,
            channel_name: curr_ch_name,
            channel_id: Some(msg.channel_id),
            message_id: Some(msg.id),
        };

        let fwd_val = serde_json::to_value(&fwd_info).ok();

        // Close the modal immediately so overlay is unmounted and clicks are never blocked
        modal_context.is_forward_modal_open.set(false);
        modal_context.forwarding_message.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::gateway::service::WsService::message_create(
                target_ch_id,
                msg.content,
                None,
                fwd_val,
            )
            .await;
        });
    };

    let msg_preview = create_memo(move || modal_context.forwarding_message.get_clone());

    view! {
        div(
            class="server-modal-overlay forward-modal-overlay",
            on:click=close,
        ) {
            div(
                class="server-modal forward-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="server-modal-header") {
                    span { "Переслать сообщение" }
                    button(class="modal-close-icon-btn", on:click=close, title="Закрыть") {
                        CloseSmallIcon()
                    }
                }

                div(class="forward-modal-body") {
                    // Preview card of the message being forwarded
                    (if let Some(msg) = msg_preview.get_clone() {
                        let snippet = if msg.content.chars().count() > 120 {
                            let s: String = msg.content.chars().take(120).collect();
                            format!("{s}…")
                        } else {
                            msg.content
                        };
                        view! {
                            div(class="forward-preview-card") {
                                div(class="preview-accent-line")
                                div(class="preview-details") {
                                    span(class="preview-author") { (msg.author_name) }
                                    p(class="preview-content") { (snippet) }
                                }
                            }
                        }
                    } else {
                        view! {}
                    })

                    // Server selection strip
                    div(class="forward-servers-section") {
                        label(class="section-title") { "ВЫБЕРИТЕ СЕРВЕР" }
                        div(class="forward-servers-strip") {
                            Keyed(
                                list=server_context.list,
                                key=|s| s.id,
                                view=move |s| {
                                    let s_id = s.id;
                                    let is_active = move || selected_server_id.get() == Some(s_id);
                                    let item_class = move || {
                                        if is_active() {
                                            "forward-server-pill active"
                                        } else {
                                            "forward-server-pill"
                                        }
                                    };
                                    view! {
                                        button(
                                            r#type="button",
                                            class=item_class,
                                            on:click=move |_| {
                                                selected_server_id.set(Some(s_id));
                                                selected_channel_id.set(None);
                                            },
                                        ) {
                                            (s.name)
                                        }
                                    }
                                }
                            )
                        }
                    }

                    // Channels list
                    div(class="forward-channels-section") {
                        label(class="section-title") { "ВЫБЕРИТЕ КАНАЛ" }
                        div(class="forward-channels-list") {
                            (if channels_list.with(|l| l.is_empty()) {
                                view! {
                                    div(class="forward-channels-empty") {
                                        "Нет доступных текстовых каналов"
                                    }
                                }
                            } else {
                                view! {
                                    Keyed(
                                        list=channels_list,
                                        key=|c| c.id,
                                        view=move |c| {
                                            let ch_id = c.id;
                                            let is_selected = move || selected_channel_id.get() == Some(ch_id);
                                            let ch_class = move || {
                                                if is_selected() {
                                                    "forward-channel-item active"
                                                } else {
                                                    "forward-channel-item"
                                                }
                                            };
                                            view! {
                                                div(
                                                    class=ch_class,
                                                    on:click=move |_| selected_channel_id.set(Some(ch_id)),
                                                ) {
                                                    span(class="channel-icon") { HashIcon() }
                                                    span(class="channel-title") { (c.name) }
                                                    (if is_selected() {
                                                        view! {
                                                            span(class="selected-check") { "✓" }
                                                        }
                                                    } else {
                                                        view! {}
                                                    })
                                                }
                                            }
                                        }
                                    )
                                }
                            })
                        }
                    }

                    div(class="modal-footer-buttons") {
                        button(class="back-btn", on:click=close) { "Отмена" }
                        button(
                            class="submit-btn forward-submit-btn",
                            disabled=move || selected_channel_id.get().is_none() || is_sending.get(),
                            on:click=handle_forward,
                        ) {
                            (if is_sending.get() {
                                "Отправка…"
                            } else {
                                "Переслать"
                            })
                        }
                    }
                }
            }
        }
    }
}
