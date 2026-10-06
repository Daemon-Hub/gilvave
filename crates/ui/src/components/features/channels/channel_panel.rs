use gilvave_core::dto::channel::ChannelType;
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::{
        common::{ChannelContext, ServerContext, UiModalContext},
        ui::icons::{ChevronDownIcon, PlusIcon},
    },
    gateway::service::WsService,
    http::api::Api,
};

use super::channel_item::ChannelItem;

#[component(inline_props)]
pub fn ChannelPanel() -> View {
    let context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let modal_context = use_context::<UiModalContext>();

    let is_text_collapsed = create_signal(false);
    let is_voice_collapsed = create_signal(false);

    let toggle_text = move |_| {
        is_text_collapsed.set(!is_text_collapsed.get());
    };

    let toggle_voice = move |_| {
        is_voice_collapsed.set(!is_voice_collapsed.get());
    };

    let on_add_text_channel = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        modal_context.create_channel_type.set(ChannelType::TEXT);
        modal_context.is_create_channel_open.set(true);
    };

    let on_add_voice_channel = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        modal_context.create_channel_type.set(ChannelType::VOICE);
        modal_context.is_create_channel_open.set(true);
    };

    let text_header_class = move || {
        if is_text_collapsed.get() {
            "channel-header collapsed"
        } else {
            "channel-header"
        }
    };

    let text_group_class = move || {
        if is_text_collapsed.get() {
            "channel-group-container collapsed"
        } else {
            "channel-group-container"
        }
    };

    let voice_header_class = move || {
        if is_voice_collapsed.get() {
            "channel-header collapsed"
        } else {
            "channel-header"
        }
    };

    let voice_group_class = move || {
        if is_voice_collapsed.get() {
            "channel-group-container collapsed"
        } else {
            "channel-group-container"
        }
    };

    let current_server_id = create_memo(move || {
        server_context
            .current
            .with(|s| s.as_ref().map(|srv| srv.id))
    });

    create_effect(move || {
        let server_id_opt = current_server_id.get();

        if let Some(channel_id) = context.current.with_untracked(|c| c.as_ref().map(|ch| ch.id)) {
            context.current.set(None);
            spawn_local_scoped(async move {
                let _ = WsService::left_channel(channel_id).await;
            });
        }

        if let Some(server_id) = server_id_opt {
            context.text.set(vec![]);
            context.voice.set(vec![]);
            spawn_local_scoped(async move {
                if let Ok(channels) = Api::get_server_channels(server_id).await {
                    let mut text_channels = Vec::new();
                    let mut voice_channels = Vec::new();
                    for channel in channels {
                        match channel.r#type {
                            ChannelType::TEXT => text_channels.push(channel),
                            ChannelType::VOICE => voice_channels.push(channel),
                        }
                    }
                    context.text.set(text_channels);
                    context.voice.set(voice_channels);
                }
            })
        } else {
            context.messages.set(vec![]);
            context.text.set(vec![]);
            context.voice.set(vec![]);
        }
    });

    view! {
        div(class="channel-panel") {
            div(class="channel-list") {
                div(
                    class=text_header_class,
                    on:click=toggle_text,
                    title="Нажмите, чтобы свернуть или развернуть",
                ) {
                    div(class="channel-header-title") {
                        span(class="channel-header-chevron") {
                            ChevronDownIcon()
                        }
                        span { "Текстовые" }
                    }
                    button(
                        class="channel-add-btn",
                        on:click=on_add_text_channel,
                        title="Создать текстовый канал",
                    ) { PlusIcon() }
                }
                div(class=text_group_class) {
                    div(class="channel-group-inner") {
                        Indexed(
                            list=context.text,
                            view=|channel| { view! {
                                ChannelItem(channel_view=channel)
                            }},
                        )
                    }
                }
            }

            div(class="channel-list") {
                div(
                    class=voice_header_class,
                    on:click=toggle_voice,
                    title="Нажмите, чтобы свернуть или развернуть",
                ) {
                    div(class="channel-header-title") {
                        span(class="channel-header-chevron") {
                            ChevronDownIcon()
                        }
                        span { "Голосовые" }
                    }
                    button(
                        class="channel-add-btn",
                        on:click=on_add_voice_channel,
                        title="Создать голосовой канал",
                    ) { PlusIcon() }
                }
                div(class=voice_group_class) {
                    div(class="channel-group-inner") {
                        Indexed(
                            list=context.voice,
                            view=|channel| { view! {
                                ChannelItem(channel_view=channel)
                            }},
                        )
                    }
                }
            }
        }
    }
}
