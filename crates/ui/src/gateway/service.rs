use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::{SinkExt, StreamExt, lock::Mutex};
use gilvave_core::{
    dto::{
        command::{CommandArgs, CommandResponse, CommandResult},
        ws::ServerSend,
    },
    error::ErrorInfo,
    ids::ChannelId,
    settings::BASE_WS_URL,
};
use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
};
use ws_stream_wasm::{WsMessage, WsMeta};

use crate::{gateway::handler::handle, utils::invoke_command};

static IS_LISTENING: AtomicBool = AtomicBool::new(false);
static SENDER: LazyLock<Arc<Mutex<Option<UnboundedSender<ServerSend>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

pub struct WsService;

impl WsService {
    pub async fn join_channel(channel_id: ChannelId) -> Result<(), ErrorInfo> {
        web_sys::console::log_1(&format!("[WS SERVICE] join_channel called: channel_id={channel_id}").into());
        let sender_ptr = SENDER.lock().await;
        match sender_ptr.as_ref() {
            Some(sender) => {
                match sender.unbounded_send(ServerSend::JoinChannel { channel_id }) {
                    Ok(_) => {
                        web_sys::console::log_1(&"[WS SERVICE] unbounded_send JoinChannel OK".into());
                        Ok(())
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("[WS SERVICE] unbounded_send JoinChannel ERR: {e}").into());
                        Err(ErrorInfo(1, e.to_string()))
                    }
                }
            }
            None => {
                web_sys::console::error_1(&"[WS SERVICE] join_channel: SENDER is None!".into());
                Err(ErrorInfo(1, "WebSocket sender not initialized".to_string()))
            }
        }
    }

    pub async fn left_channel(channel_id: ChannelId) -> Result<(), ErrorInfo> {
        web_sys::console::log_1(&format!("[WS SERVICE] left_channel called: channel_id={channel_id}").into());
        let sender_ptr = SENDER.lock().await;
        match sender_ptr.as_ref() {
            Some(sender) => sender
                .unbounded_send(ServerSend::LeftChannel { channel_id })
                .map_err(|e| ErrorInfo(1, e.to_string())),
            None => Err(ErrorInfo(1, "WebSocket sender not initialized".to_string())),
        }
    }

    // ─── VOICE CALL SIGNALING (WS) ───────────────────────────────────────────

    // TODO [WS]: Отправка ServerSend::JoinVoiceChannel { channel_id }
    // Оповещает шлюз о присоединении пользователя к голосовому каналу
    // pub async fn join_voice_channel(channel_id: ChannelId) -> Result<(), ErrorInfo>

    // TODO [WS]: Отправка ServerSend::LeftVoiceChannel { channel_id }
    // Оповещает шлюз об отключении пользователя от голосового канала
    // pub async fn left_voice_channel(channel_id: ChannelId) -> Result<(), ErrorInfo>

    // TODO [WS]: Отправка ServerSend::VoiceStateUpdate { channel_id, is_muted, is_deafened }
    // Синхронизация статуса микрофона / звука со всеми клиентами комнаты в реальном времени
    // pub async fn update_voice_state(channel_id: ChannelId, is_muted: bool, is_deafened: bool) -> Result<(), ErrorInfo>

    // TODO [WS]: Отправка ServerSend::VoiceSignal { channel_id, data }
    // Обмен сигнальными сообщениями WebRTC (SDP Offer, SDP Answer, ICE Candidates) через WebSocket
    // pub async fn send_voice_signal(channel_id: ChannelId, data: String) -> Result<(), ErrorInfo>

    pub async fn message_create(
        channel_id: ChannelId,
        content: String,
    ) -> Result<(), ErrorInfo> {
        web_sys::console::log_1(&format!("[WS SERVICE] message_create called: channel_id={channel_id}, content={content}").into());
        let sender_ptr = SENDER.lock().await;
        match sender_ptr.as_ref() {
            Some(sender) => {
                match sender.unbounded_send(ServerSend::MessageCreate {
                    channel_id,
                    content,
                }) {
                    Ok(_) => {
                        web_sys::console::log_1(&"[WS SERVICE] unbounded_send MessageCreate OK".into());
                        Ok(())
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("[WS SERVICE] unbounded_send MessageCreate ERR: {e}").into());
                        Err(ErrorInfo(1, e.to_string()))
                    }
                }
            }
            None => {
                web_sys::console::error_1(&"[WS SERVICE] message_create: SENDER is None!".into());
                Err(ErrorInfo(1, "WebSocket sender not initialized".to_string()))
            }
        }
    }

    pub async fn channel_history_before(
        channel_id: ChannelId,
        timestamp: time::OffsetDateTime,
    ) -> Result<(), ErrorInfo> {
        web_sys::console::log_1(&format!("[WS SERVICE] channel_history_before called: channel_id={channel_id}").into());
        let sender_ptr = SENDER.lock().await;
        match sender_ptr.as_ref() {
            Some(sender) => sender
                .unbounded_send(ServerSend::ChannelHistoryBefore {
                    channel_id,
                    timestamp,
                })
                .map_err(|e| ErrorInfo(1, e.to_string())),
            None => Err(ErrorInfo(1, "WebSocket sender not initialized".to_string())),
        }
    }

    pub async fn channel_history_after(
        channel_id: ChannelId,
        timestamp: time::OffsetDateTime,
    ) -> Result<(), ErrorInfo> {
        web_sys::console::log_1(&format!("[WS SERVICE] channel_history_after called: channel_id={channel_id}").into());
        let sender_ptr = SENDER.lock().await;
        match sender_ptr.as_ref() {
            Some(sender) => sender
                .unbounded_send(ServerSend::ChannelHistoryAfter {
                    channel_id,
                    timestamp,
                })
                .map_err(|e| ErrorInfo(1, e.to_string())),
            None => Err(ErrorInfo(1, "WebSocket sender not initialized".to_string())),
        }
    }

    pub async fn listen_web_socket() -> Result<bool, ErrorInfo> {
        if IS_LISTENING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            web_sys::console::warn_1(&"[WS SERVICE] listen_web_socket is already running! Skipping duplicate call.".into());
            return Ok(true);
        }

        wasm_bindgen_futures::spawn_local(async {
            loop {
                web_sys::console::log_1(&"[WS SERVICE] listen_web_socket loop start".into());

                let token = match invoke_command(CommandArgs::GetAccessToken.to_json()).await {
                    CommandResult::Ok(CommandResponse::GetAccessToken(t)) => t,
                    _ => String::new(),
                };
                let ws_url = if token.is_empty() {
                    BASE_WS_URL.to_string()
                } else {
                    format!("{BASE_WS_URL}?token={token}")
                };

                let (sender, mut receiver) = unbounded::<ServerSend>();

                {
                    let mut state_sender = SENDER.lock().await;
                    *state_sender = Some(sender);
                }

                web_sys::console::log_1(&format!("[WS SERVICE] connecting to {ws_url}").into());
                match WsMeta::connect(&ws_url, None).await {
                    Ok((_meta, ws_stream)) => {
                        web_sys::console::log_1(&"[WS SERVICE] connected successfully!".into());
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        let receive_task = async {
                            web_sys::console::log_1(&"[WS SERVICE] receive_task start".into());
                            while let Some(msg) = ws_receiver.next().await {
                                match msg {
                                    WsMessage::Text(text) => {
                                        web_sys::console::log_1(&format!("[WS SERVICE RECV TEXT] {text}").into());
                                        handle(text).await;
                                    }
                                    WsMessage::Binary(bin) => {
                                        web_sys::console::log_1(&format!("[WS SERVICE RECV BIN] len={}", bin.len()).into());
                                    }
                                }
                            }
                            web_sys::console::warn_1(&"[WS SERVICE] receive_task ended".into());
                        };

                        let send_task = async {
                            web_sys::console::log_1(&"[WS SERVICE] send_task start".into());
                            while let Some(msg) = receiver.next().await {
                                match serde_json::to_string(&msg) {
                                    Ok(json) => {
                                        web_sys::console::log_1(&format!("[WS SERVICE SENDING] {json}").into());
                                        if let Err(e) = ws_sender.send(WsMessage::Text(json.clone())).await {
                                            web_sys::console::error_1(&format!("[WS SERVICE SEND ERR] {e:?}, json={json}").into());
                                            break;
                                        } else {
                                            web_sys::console::log_1(&format!("[WS SERVICE SEND OK] {json}").into());
                                        }
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(&format!("[WS SERVICE JSON SERIALIZE ERR] {e:?}").into());
                                    }
                                }
                            }
                            web_sys::console::warn_1(&"[WS SERVICE] send_task ended".into());
                        };

                        futures_util::future::select(Box::pin(receive_task), Box::pin(send_task)).await;
                        web_sys::console::warn_1(&"[WS SERVICE] one of ws tasks finished, closing and reconnecting...".into());

                        gloo_timers::future::sleep(core::time::Duration::from_millis(1000)).await;
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("[WS SERVICE] WebSocket connection error: {e:?}").into());
                        let _ = crate::http::api::Api::update_tokens().await;
                        gloo_timers::future::sleep(core::time::Duration::from_secs(2)).await;
                        continue;
                    }
                }
            }
        });

        Ok(true)
    }
}
