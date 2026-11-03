//! Voice channel interactive area with pre-connect stage and connected participant room dock.

mod header;
mod room_dock;
mod welcome_card;

pub(crate) use header::VoiceHeader;
pub(crate) use room_dock::ConnectedRoom;
pub(crate) use welcome_card::PreConnectCard;

use sycamore::prelude::*;
use crate::components::common::{classes, ChannelContext, ServerContext, UserProfileContext};

#[component]
pub fn VoiceChannelArea() -> View {
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = use_context::<UserProfileContext>();

    let is_connected = create_signal(false);

    // Reset connected state when switching channels
    create_effect(move || {
        channel_context.current.track();
        is_connected.set(false);
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
        is_connected.set(true);
        // TODO [HTTP]: POST /channels/{channel_id}/voice/join (SFU token & ICE credentials)
        // TODO [WS]: ServerSend::JoinVoiceChannel { channel_id }
        // TODO [WebRTC]: navigator.mediaDevices.getUserMedia(...) & RTCPeerConnection SDP
    };

    let on_disconnect = move |_| {
        is_connected.set(false);
        // TODO [WS]: ServerSend::LeftVoiceChannel { channel_id }
        // TODO [HTTP]: POST /channels/{channel_id}/voice/leave
        // TODO [WebRTC]: Stop audio tracks and close RTCPeerConnection
    };

    let on_toggle_mic = move |_| {
        let new_muted = !user_profile.is_muted.get();
        user_profile.is_muted.set(new_muted);
        // TODO [WS]: ServerSend::VoiceStateUpdate { channel_id, is_muted: new_muted, ... }
    };

    let on_toggle_deafen = move |_| {
        let new_deafened = !user_profile.is_deafened.get();
        user_profile.is_deafened.set(new_deafened);
        // TODO [WS]: ServerSend::VoiceStateUpdate { channel_id, is_deafened: new_deafened, ... }
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
