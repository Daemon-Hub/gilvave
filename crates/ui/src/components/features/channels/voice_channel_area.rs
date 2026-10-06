use sycamore::prelude::*;

use crate::components::{
    common::{classes, ChannelContext, ServerContext, UserProfileContext},
    ui::icons::{
        HeadphonesIcon, HeadphonesOffIcon, MicIcon, MicOffIcon, PhoneIcon, PhoneOffIcon,
        Volume2Icon,
    },
};

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
        channel_context
            .current
            .with(|c| c.as_ref().map_or_else(|| "Голосовой канал".into(), |ch| ch.name.clone()))
    });

    let server_name = create_memo(move || {
        server_context
            .current
            .with(|s| s.as_ref().map_or_else(|| String::new(), |srv| srv.name.clone()))
    });

    // TODO [HTTP]: GET /channels/{channel_id}/voice/participants — первоначальный запрос списка активных участников голосового канала
    // TODO [WS]: listen::<VoiceParticipantEvent>("voice_participant_joined") — добавление нового подключившегося пользователя в сетку участников
    // TODO [WS]: listen::<VoiceParticipantEvent>("voice_participant_left") — удаление отключившегося пользователя из сетки
    // TODO [WS]: listen::<VoiceSpeakingEvent>("voice_speaking") — VAD (индикатор речи, подсвечивающий speaking-pulse-ring вокруг аватара)

    let on_back = move |_| {
        channel_context.current.set(None);
    };

    let on_connect = move |_| {
        is_connected.set(true);
        // TODO [HTTP]: POST /channels/{channel_id}/voice/join
        // Запрос к серверу на получение токена звонка (LiveKit/mediasoup/WebRTC SFU room token) и адресов ICE/STUN/TURN серверов.
        //
        // TODO [WS]: Отправка события через WebSocket gateway: ServerSend::JoinVoiceChannel { channel_id }
        // Оповещает всех участников сервера о подключении данного пользователя к голосовому каналу.
        //
        // TODO [WebRTC / Audio]: Инициализация медиа-устройств:
        // 1. web_sys::window().navigator().media_devices().get_user_media(...) для захвата микрофона.
        // 2. Установка RTCPeerConnection и отправка локального SDP Offer/Answer через WS сигнальный шлюз (ServerSend::VoiceSignal).
    };

    let on_disconnect = move |_| {
        is_connected.set(false);
        // TODO [WS]: Отправка события через WebSocket gateway: ServerSend::LeftVoiceChannel { channel_id }
        // Оповещает других участников сервера о том, что пользователь покинул голосовую комнату.
        //
        // TODO [HTTP]: POST /channels/{channel_id}/voice/leave
        // Уведомление бэкенда о завершении голосовой сессии и освобождении выделенных медиа-ресурсов SFU.
        //
        // TODO [WebRTC / Audio]: Остановка всех аудио-треков:
        // stream.get_tracks().for_each(|t| t.stop()) и peer_connection.close().
    };

    let on_toggle_mic = move |_| {
        let new_muted = !user_profile.is_muted.get();
        user_profile.is_muted.set(new_muted);
        // TODO [WS]: Отправка события через WebSocket gateway: ServerSend::VoiceStateUpdate { channel_id, is_muted: new_muted, is_deafened: user_profile.is_deafened.get() }
        // Синхронизирует состояние микрофона (Mute/Unmute) с остальными участниками звонка и отключает передачу RTP-аудиопотока на SFU.
    };

    let on_toggle_deafen = move |_| {
        let new_deafened = !user_profile.is_deafened.get();
        user_profile.is_deafened.set(new_deafened);
        // TODO [WS]: Отправка события через WebSocket gateway: ServerSend::VoiceStateUpdate { channel_id, is_muted: user_profile.is_muted.get(), is_deafened: new_deafened }
        // Сервер перестает направлять входящие аудио-потоки от других участников к данному клиенту.
    };

    let user_avatar_view = move || {
        let av_opt = user_profile
            .avatar
            .with(|av| (!av.is_empty()).then(|| av.clone()));
        if let Some(av) = av_opt {
            view! {
                img(src=av, alt="", class="participant-avatar-img")
            }
        } else {
            let initial = user_profile.username.with(|name| {
                name.chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string()
            });
            view! {
                div(class="participant-avatar-placeholder") {
                    span { (initial) }
                }
            }
        }
    };

    view! {
        div(class="voice-channel-area") {
            // Header
            div(class="voice-header") {
                button(class="chat-back-btn", on:click=on_back) {
                    svg(
                        xmlns="http://www.w3.org/2000/svg",
                        width="18",
                        height="18",
                        viewBox="0 0 24 24",
                        fill="none",
                        stroke="currentColor",
                        stroke-width="2.5",
                        stroke-linecap="round",
                        stroke-linejoin="round",
                    ) {
                        polyline(points="15 18 9 12 15 6")
                    }
                    span { "Каналы" }
                }

                div(class="voice-header-main") {
                    div(class="voice-header-title") {
                        span(class="voice-channel-icon") { Volume2Icon() }
                        span(class="voice-channel-name") { (channel_name) }
                    }

                    span(
                        class=classes(vec![
                            "voice-header-badge".into(),
                            ("connected", is_connected.into()).into(),
                        ])
                    ) {
                        (if is_connected.get() {
                            "Подключено • RTC 24 ms"
                        } else {
                            "Голосовой канал"
                        })
                    }
                }

                div(class="voice-header-server-info") {
                    span { (server_name) }
                }
            }

            // Body
            div(
                class=classes(vec![
                    "voice-body".into(),
                    ("connected", is_connected.into()).into(),
                ])
            ) {
                (if is_connected.get() {
                    view! {
                        // Connected State: Participant room & floating controls
                        div(class="voice-connected-room") {
                            div(class="voice-participants-stage") {
                                div(class="voice-participant-card") {
                                    div(class="participant-avatar-wrapper") {
                                        div(
                                            class=classes(vec![
                                                "speaking-pulse-ring".into(),
                                                ("muted", user_profile.is_muted.into()).into(),
                                            ])
                                        ) {}
                                        (user_avatar_view())
                                    }

                                    div(class="participant-meta") {
                                        span(class="participant-username") {
                                            (user_profile.username.get_clone())
                                        }

                                        div(class="participant-status-badges") {
                                            (if user_profile.is_muted.get() {
                                                view! {
                                                    span(class="status-pill muted", title="Микрофон выключен") {
                                                        MicOffIcon()
                                                        span { "Без микрофона" }
                                                    }
                                                }
                                            } else {
                                                view! {
                                                    span(class="status-pill active", title="Микрофон активен") {
                                                        MicIcon()
                                                        span { "В эфире" }
                                                    }
                                                }
                                            })

                                            (if user_profile.is_deafened.get() {
                                                view! {
                                                    span(class="status-pill deafened", title="Звук выключен") {
                                                        HeadphonesOffIcon()
                                                        span { "Без звука" }
                                                    }
                                                }
                                            } else {
                                                view! {}
                                            })
                                        }
                                    }
                                }
                            }

                            // Bottom Controls Dock
                            div(class="voice-dock-controls") {
                                button(
                                    class=classes(vec![
                                        "voice-control-btn".into(),
                                        ("off", user_profile.is_muted.into()).into(),
                                    ]),
                                    on:click=on_toggle_mic,
                                    title=if user_profile.is_muted.get() { "Включить микрофон" } else { "Выключить микрофон" },
                                ) {
                                    (if user_profile.is_muted.get() {
                                        view! { MicOffIcon() }
                                    } else {
                                        view! { MicIcon() }
                                    })
                                    span {
                                        (if user_profile.is_muted.get() {
                                            "Вкл. микрофон"
                                        } else {
                                            "Микрофон"
                                        })
                                    }
                                }

                                button(
                                    class=classes(vec![
                                        "voice-control-btn".into(),
                                        ("off", user_profile.is_deafened.into()).into(),
                                    ]),
                                    on:click=on_toggle_deafen,
                                    title=if user_profile.is_deafened.get() { "Включить звук" } else { "Выключить звук" },
                                ) {
                                    (if user_profile.is_deafened.get() {
                                        view! { HeadphonesOffIcon() }
                                    } else {
                                        view! { HeadphonesIcon() }
                                    })
                                    span {
                                        (if user_profile.is_deafened.get() {
                                            "Вкл. звук"
                                        } else {
                                            "Звук"
                                        })
                                    }
                                }

                                button(
                                    class="voice-control-btn disconnect-btn",
                                    on:click=on_disconnect,
                                    title="Покинуть голосовой канал",
                                ) {
                                    PhoneOffIcon()
                                    span { "Отключиться" }
                                }
                            }
                        }
                    }
                } else {
                    view! {
                        // Pre-connect Welcome Screen
                        div(class="voice-welcome-card") {
                            div(class="voice-radar-pulse") {
                                div(class="radar-ring ring-1") {}
                                div(class="radar-ring ring-2") {}
                                div(class="radar-center") {
                                    Volume2Icon()
                                }
                            }

                            h2(class="voice-welcome-title") { (channel_name) }
                            p(class="voice-welcome-desc") {
                                "Вы выбрали голосовой канал. Подключитесь, чтобы разговаривать с участниками сервера и слушать других."
                            }

                            div(class="voice-specs-row") {
                                div(class="spec-item") {
                                    span(class="spec-icon") {
                                        (if user_profile.is_muted.get() {
                                            view! { MicOffIcon() }
                                        } else {
                                            view! { MicIcon() }
                                        })
                                    }
                                    span {
                                        (if user_profile.is_muted.get() {
                                            "Микрофон выключен"
                                        } else {
                                            "Микрофон готов"
                                        })
                                    }
                                }
                                div(class="spec-item") {
                                    span(class="spec-icon") { HeadphonesIcon() }
                                    span { "Стерео звук 64 kbps" }
                                }
                            }

                            button(class="voice-connect-action-btn", on:click=on_connect) {
                                PhoneIcon()
                                span { "Подключиться к голосовому каналу" }
                            }
                        }
                    }
                })
            }
        }
    }
}
