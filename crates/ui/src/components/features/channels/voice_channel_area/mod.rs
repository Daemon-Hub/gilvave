//! Voice channel interactive area with pre-connect stage and connected participant room dock.

mod header;
mod room_dock;
mod welcome_card;

pub(crate) use header::VoiceHeader;
pub(crate) use room_dock::ConnectedRoom;
pub(crate) use welcome_card::PreConnectCard;

use crate::components::common::{
    ChannelContext, ServerContext, UserProfileContext, VoiceConnectionStatus, VoiceContext, classes,
};
use crate::voice::VoiceManager;
use sycamore::prelude::*;

#[component]
pub fn VoiceChannelArea() -> View {
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = use_context::<UserProfileContext>();
    let voice_context = use_context::<VoiceContext>();

    let is_connected = create_signal(false);

    create_effect(move || {
        let current_cid = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
        let voice_cid = voice_context.current_channel_id.get();
        let status = voice_context.connection_status.get();
        let connected = current_cid.is_some()
            && current_cid == voice_cid
            && status == VoiceConnectionStatus::Connected;
        is_connected.set(connected);
    });

    let channel_name = create_memo(move || {
        channel_context.current.with(|c| {
            c.as_ref()
                .map_or_else(|| "Голосовой канал".into(), |ch| ch.name.clone())
        })
    });

    let server_name = create_memo(move || {
        server_context
            .current
            .with(|s| s.as_ref().map_or_else(String::new, |srv| srv.name.clone()))
    });

    let on_back = move |_| {
        channel_context.current.set(None);
    };

    let on_connect = move |_| {
        if let Some(ch) = channel_context.current.get_clone() {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = VoiceManager::connect(ch.id, voice_context, user_profile).await {
                    web_sys::console::error_1(&format!("[VOICE] Connect error: {e}").into());
                }
            });
        }
    };

    let on_disconnect = move |_| {
        let uid = user_profile.user_id.get();
        if let Some(uid) = uid {
            voice_context.channel_participants.update(|map| {
                for list in map.values_mut() {
                    list.retain(|p| p.user_id != uid);
                }
            });
        }
        wasm_bindgen_futures::spawn_local(async move {
            VoiceManager::disconnect(voice_context, uid).await;
        });
    };

    let on_toggle_mic = move |_| {
        let new_muted = !user_profile.is_muted.get();
        VoiceManager::set_muted(new_muted, voice_context, user_profile);
    };

    let on_toggle_deafen = move |_| {
        let new_deafened = !user_profile.is_deafened.get();
        VoiceManager::set_deafened(new_deafened, voice_context, user_profile);
    };

    let on_toggle_camera = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = VoiceManager::toggle_camera(voice_context).await {
                web_sys::console::error_1(&format!("[VOICE] Camera toggle error: {e}").into());
            }
        });
    };

    let on_toggle_screen = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = VoiceManager::toggle_screen_share(voice_context).await {
                web_sys::console::error_1(&format!("[VOICE] Screen share error: {e}").into());
            }
        });
    };

    view! {
        div(class="voice-channel-area") {
            VoiceHeader(
                channel_name=channel_name,
                server_name=server_name,
                is_connected=is_connected,
                on_back=on_back,
            )

            div(
                class=classes(vec![
                    "voice-body".into(),
                    ("connected", is_connected.into()).into(),
                ])
            ) {
                (if is_connected.get() {
                    view! {
                        ConnectedRoom(
                            on_toggle_mic=on_toggle_mic,
                            on_toggle_deafen=on_toggle_deafen,
                            on_toggle_camera=on_toggle_camera,
                            on_toggle_screen=on_toggle_screen,
                            on_disconnect=on_disconnect,
                        )
                    }
                } else {
                    view! {
                        PreConnectCard(
                            channel_name=channel_name,
                            on_connect=on_connect,
                        )
                    }
                })
            }
        }
    }
}
