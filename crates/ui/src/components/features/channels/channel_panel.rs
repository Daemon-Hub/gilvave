use sycamore::{futures::spawn_local_scoped, prelude::*};

use super::channel_item::ChannelItem;
use crate::{
    cache::CacheService,
    components::{
        common::{
            ChannelContext, ServerContext, UiModalContext, UserProfileContext,
            VoiceConnectionStatus, VoiceContext,
        },
        ui::icons::{ChevronDownIcon, PhoneOffIcon, PlusIcon},
    },
    gateway::service::WsService,
    http::api::Api,
};
use gilvave_core::dto::servers::channels::ChannelType;

#[component(inline_props)]
pub fn ChannelPanel() -> View {
    let context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let modal_context = use_context::<UiModalContext>();
    let user_profile = use_context::<UserProfileContext>();
    let voice_context = use_context::<VoiceContext>();

    let is_voice_connected = create_memo(move || {
        voice_context.connection_status.get() == VoiceConnectionStatus::Connected
    });

    let active_voice_channel_name = create_memo(move || {
        let active_cid = voice_context.current_channel_id.get();
        context.voice.with(|channels| {
            channels
                .iter()
                .find(|ch| Some(ch.id) == active_cid)
                .map(|ch| ch.name.clone())
                .unwrap_or_else(|| "Голосовой канал".to_string())
        })
    });

    let on_disconnect_voice = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let uid = user_profile.user_id.get();
        if let Some(uid) = uid {
            voice_context.channel_participants.update(|map| {
                for list in map.values_mut() {
                    list.retain(|p| p.user_id != uid);
                }
            });
        }
        wasm_bindgen_futures::spawn_local(async move {
            crate::voice::VoiceManager::disconnect(voice_context, uid).await;
        });
    };

    let on_open_active_voice = move |_| {
        let active_cid = voice_context.current_channel_id.get();
        if let Some(ch) = context.voice.with(|channels| {
            channels
                .iter()
                .find(|ch| Some(ch.id) == active_cid)
                .cloned()
        }) {
            context.current.set(Some(ch));
        }
    };

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

        if let Some(channel_id) = context
            .current
            .with_untracked(|c| c.as_ref().map(|ch| ch.id))
        {
            context.current.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = WsService::left_channel(channel_id).await;
            });
        }

        if let Some(server_id) = server_id_opt {
            context.text.set(vec![]);
            context.voice.set(vec![]);
            spawn_local_scoped(async move {
                let cached = CacheService::get_cached_channels(server_id).await;
                if !cached.is_empty() {
                    let mut text_channels = Vec::new();
                    let mut voice_channels = Vec::new();
                    for channel in cached {
                        match channel.r#type {
                            ChannelType::TEXT => text_channels.push(channel),
                            ChannelType::VOICE => voice_channels.push(channel),
                        }
                    }
                    context.text.set(text_channels);
                    context.voice.set(voice_channels);
                }

                if let Ok(channels) = Api::get_server_channels(server_id).await {
                    CacheService::sync_cached_channels(server_id, channels.clone()).await;
                    let mut text_channels = Vec::with_capacity(channels.len());
                    let mut voice_channels = Vec::with_capacity(channels.len());
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

            (if is_voice_connected.get() {
                view! {
                    div(class="voice-status-bar") {
                        div(class="voice-status-info", on:click=on_open_active_voice) {
                            div(class="voice-status-indicator") {}
                            div(class="voice-status-text") {
                                span(class="voice-status-title") { "Голос подключен" }
                                span(class="voice-status-channel") { (active_voice_channel_name.get_clone()) }
                            }
                        }
                        div(class="voice-status-actions") {
                            button(
                                class="voice-status-btn disconnect",
                                on:click=on_disconnect_voice,
                                title="Покинуть звонок",
                            ) {
                                PhoneOffIcon()
                            }
                        }
                    }
                }
            } else {
                view! {}
            })
        }
    }
}
