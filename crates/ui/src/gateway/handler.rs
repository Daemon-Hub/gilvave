use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use std::sync::{Arc, LazyLock, Mutex};

use gilvave_core::{
    dto::{
        messages::MessageView,
        voice::{VoiceParticipantView, VoiceSignalData},
        ws::ServerRecieve,
    },
    ids::{ChannelId, MessageId, ServerId, UserId},
};

#[derive(Clone, Debug)]
pub enum GatewayEvent {
    Connected,
    Disconnected,
    MessageNew(MessageView),
    MessageUpdate(MessageView),
    MessageDelete {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    ChannelHistoryBefore(Vec<MessageView>),
    ChannelHistoryAfter(Vec<MessageView>),
    ServerOnline {
        server_id: ServerId,
        online_users: Vec<UserId>,
    },
    UserOnline {
        user_id: UserId,
    },
    UserOffline {
        user_id: UserId,
    },
    VoiceRoomJoined {
        channel_id: ChannelId,
        participants: Vec<VoiceParticipantView>,
    },
    VoiceParticipantJoined {
        channel_id: ChannelId,
        participant: VoiceParticipantView,
    },
    VoiceParticipantLeft {
        channel_id: ChannelId,
        user_id: UserId,
    },
    VoiceStateUpdate {
        channel_id: ChannelId,
        user_id: UserId,
        is_muted: bool,
        is_deafened: bool,
    },
    VoiceSignal {
        channel_id: ChannelId,
        signal: VoiceSignalData,
    },
    VoiceSpeaking {
        channel_id: ChannelId,
        user_id: UserId,
        is_speaking: bool,
    },
    VoiceVideoState {
        channel_id: ChannelId,
        user_id: UserId,
        is_camera_on: bool,
        is_screen_sharing: bool,
    },
}

static SUBSCRIBERS: LazyLock<Arc<Mutex<Vec<UnboundedSender<GatewayEvent>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

pub fn subscribe_gateway_events() -> UnboundedReceiver<GatewayEvent> {
    let (tx, rx) = unbounded();
    if let Ok(mut subs) = SUBSCRIBERS.lock() {
        subs.push(tx);
    }
    rx
}

pub fn dispatch_gateway_event(event: GatewayEvent) {
    if let Ok(mut subs) = SUBSCRIBERS.lock() {
        subs.retain(|tx| tx.unbounded_send(event.clone()).is_ok());
    }
}

pub async fn handle(text: &str) {
    match serde_json::from_str::<ServerRecieve>(text) {
        Ok(msg) => match msg {
            ServerRecieve::HeartbeatAck => {
                web_sys::console::log_1(&"[WS] HeartbeatAck".into());
            }
            ServerRecieve::Hello => {
                web_sys::console::log_1(&"[WS] Hello received".into());
            }
            ServerRecieve::Error { message } => {
                web_sys::console::error_1(&format!("[WS] Server error: {message}").into());
            }
            ServerRecieve::JoinSuccess => {
                web_sys::console::log_1(&"[WS] JoinSuccess received".into());
            }
            ServerRecieve::MessageNew(message_view) => {
                web_sys::console::log_1(
                    &format!("[WS] MessageNew received: {:?}", message_view).into(),
                );
                #[cfg(target_arch = "wasm32")]
                {
                    let mv_clone = message_view.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        crate::cache::CacheService::save_cached_message(mv_clone).await;
                    });
                }
                dispatch_gateway_event(GatewayEvent::MessageNew(message_view));
            }
            ServerRecieve::MessageUpdate(message_view) => {
                web_sys::console::log_1(
                    &format!("[WS] MessageUpdate received: {:?}", message_view).into(),
                );
                dispatch_gateway_event(GatewayEvent::MessageUpdate(message_view));
            }
            ServerRecieve::MessageDelete {
                channel_id,
                message_id,
            } => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] MessageDelete received: channel_id={channel_id}, message_id={message_id}"
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::MessageDelete {
                    channel_id,
                    message_id,
                });
            }
            ServerRecieve::ChannelHistoryBefore(messages) => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] ChannelHistoryBefore received {} messages",
                        messages.len()
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::ChannelHistoryBefore(messages));
            }
            ServerRecieve::ChannelHistoryAfter(messages) => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] ChannelHistoryAfter received {} messages",
                        messages.len()
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::ChannelHistoryAfter(messages));
            }
            ServerRecieve::ServerOnline {
                server_id,
                online_users,
            } => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] ServerOnline received for server_id={server_id}, count={}",
                        online_users.len()
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::ServerOnline {
                    server_id,
                    online_users,
                });
            }
            ServerRecieve::UserOnline { user_id } => {
                web_sys::console::log_1(&format!("[WS] UserOnline received: {user_id}").into());
                dispatch_gateway_event(GatewayEvent::UserOnline { user_id });
            }
            ServerRecieve::UserOffline { user_id } => {
                web_sys::console::log_1(&format!("[WS] UserOffline received: {user_id}").into());
                dispatch_gateway_event(GatewayEvent::UserOffline { user_id });
            }
            ServerRecieve::VoiceRoomJoined {
                channel_id,
                participants,
            } => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] VoiceRoomJoined received for channel_id={channel_id}, count={}",
                        participants.len()
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::VoiceRoomJoined {
                    channel_id,
                    participants,
                });
            }
            ServerRecieve::VoiceParticipantJoined {
                channel_id,
                participant,
            } => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] VoiceParticipantJoined received for channel_id={channel_id}, user_id={}",
                        participant.user_id
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::VoiceParticipantJoined {
                    channel_id,
                    participant,
                });
            }
            ServerRecieve::VoiceParticipantLeft {
                channel_id,
                user_id,
            } => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] VoiceParticipantLeft received for channel_id={channel_id}, user_id={user_id}"
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::VoiceParticipantLeft {
                    channel_id,
                    user_id,
                });
            }
            ServerRecieve::VoiceStateUpdate {
                channel_id,
                user_id,
                is_muted,
                is_deafened,
            } => {
                web_sys::console::log_1(
                    &format!(
                        "[WS] VoiceStateUpdate received: channel_id={channel_id}, user_id={user_id}, muted={is_muted}, deafened={is_deafened}"
                    )
                    .into(),
                );
                dispatch_gateway_event(GatewayEvent::VoiceStateUpdate {
                    channel_id,
                    user_id,
                    is_muted,
                    is_deafened,
                });
            }
            ServerRecieve::VoiceSignal { channel_id, signal } => {
                web_sys::console::log_1(
                    &format!("[WS] VoiceSignal received for channel_id={channel_id}").into(),
                );
                dispatch_gateway_event(GatewayEvent::VoiceSignal { channel_id, signal });
            }
            ServerRecieve::VoiceSpeaking {
                channel_id,
                user_id,
                is_speaking,
            } => {
                dispatch_gateway_event(GatewayEvent::VoiceSpeaking {
                    channel_id,
                    user_id,
                    is_speaking,
                });
            }
            ServerRecieve::VoiceVideoState {
                channel_id,
                user_id,
                is_camera_on,
                is_screen_sharing,
            } => {
                dispatch_gateway_event(GatewayEvent::VoiceVideoState {
                    channel_id,
                    user_id,
                    is_camera_on,
                    is_screen_sharing,
                });
            }
        },
        Err(e) => {
            web_sys::console::error_1(
                &format!("[WS] parse ServerRecieve error: {e}, text: {text}").into(),
            );
        }
    }
}
