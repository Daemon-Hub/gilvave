//! Connected voice channel room stage with participant cards, screen share theater, adaptive grid, and floating dock controls.

use crate::components::{
    common::{UserProfileContext, VoiceContext, classes},
    ui::icons::{
        FullscreenIcon, GearIcon, HeadphonesIcon, HeadphonesOffIcon, MicIcon, MicOffIcon,
        PhoneOffIcon, PinIcon, ScreenShareIcon, ScreenShareOffIcon, VideoIcon, VideoOffIcon,
    },
};
use gilvave_core::dto::voice::{VideoCodecPreference, VoiceParticipantView};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

fn toggle_element_fullscreen(node_ref: NodeRef) {
    if let Some(elem) = node_ref
        .try_get()
        .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        && let Some(doc) = web_sys::window().and_then(|w| w.document())
    {
        if doc.fullscreen_element().is_some() {
            doc.exit_fullscreen();
        } else {
            let _ = elem.request_fullscreen();
        }
    }
}

#[component(inline_props)]
fn VideoTile(stream: web_sys::MediaStream, is_mirrored: bool) -> View {
    let video_ref = create_node_ref();
    let stream_clone = stream.clone();

    create_effect(move || {
        if let Some(video) = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
        {
            video.set_muted(true);
            video.set_src_object(Some(&stream_clone));
            let v_clone = video.clone();
            let on_loaded = Closure::<dyn Fn()>::new(move || {
                let _ = v_clone.play();
            });
            video.set_onloadedmetadata(Some(on_loaded.as_ref().unchecked_ref()));
            on_loaded.forget();

            if let Ok(promise) = video.play() {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                });
            }
        }
    });

    on_cleanup(move || {
        if let Some(video) = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
        {
            let _ = video.pause();
            video.set_src_object(None);
            video.set_onloadedmetadata(None);
        }
    });

    view! {
        video(
            r#ref=video_ref,
            autoplay=true,
            playsinline=true,
            muted=true,
            class=classes(vec![
                "video-tile-element".into(),
                "participant-video-stream".into(),
                ("mirrored", is_mirrored.into()).into(),
            ]),
        )
    }
}

#[component(inline_props)]
fn ParticipantCard(
    participant: VoiceParticipantView,
    is_spotlight: bool,
    is_filmstrip: bool,
) -> View {
    let user_profile = use_context::<UserProfileContext>();
    let voice_ctx = use_context::<VoiceContext>();
    let card_ref = create_node_ref();

    let user_id = participant.user_id;
    let is_me = user_profile.user_id.get() == Some(user_id);
    let username = if is_me {
        user_profile.username.get_clone()
    } else {
        participant.username.clone()
    };
    let avatar = if is_me && !user_profile.avatar.get_clone().is_empty() {
        user_profile.avatar.get_clone()
    } else {
        participant.avatar.clone()
    };
    let initial = username
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    let p_muted = participant.is_muted;
    let p_deafened = participant.is_deafened;
    let p_speaking = participant.is_speaking;
    let p_camera_on = participant.is_camera_on;
    let p_screen_sharing = participant.is_screen_sharing;

    let live_participant = create_memo(move || {
        if is_me {
            None
        } else {
            voice_ctx
                .participants
                .with(|list| list.iter().find(|p| p.user_id == user_id).cloned())
        }
    });

    let is_muted = create_memo(move || {
        if is_me {
            user_profile.is_muted.get()
        } else {
            live_participant
                .with(|p| p.as_ref().map(|v| v.is_muted))
                .unwrap_or(p_muted)
        }
    });

    let is_deafened = create_memo(move || {
        if is_me {
            user_profile.is_deafened.get()
        } else {
            live_participant
                .with(|p| p.as_ref().map(|v| v.is_deafened))
                .unwrap_or(p_deafened)
        }
    });

    let is_speaking = create_memo(move || {
        if is_me {
            voice_ctx.is_speaking.get() && !is_muted.get()
        } else {
            live_participant
                .with(|p| p.as_ref().map(|v| v.is_speaking))
                .unwrap_or(p_speaking)
                && !is_muted.get()
        }
    });

    let is_camera_on = create_memo(move || {
        if is_me {
            voice_ctx.is_camera_on.get()
        } else {
            live_participant
                .with(|p| p.as_ref().map(|v| v.is_camera_on))
                .unwrap_or(p_camera_on)
        }
    });

    let is_screen_sharing = create_memo(move || {
        if is_me {
            voice_ctx.is_screen_sharing.get()
        } else {
            live_participant
                .with(|p| p.as_ref().map(|v| v.is_screen_sharing))
                .unwrap_or(p_screen_sharing)
        }
    });

    let cam_stream = create_memo(move || {
        if is_me {
            voice_ctx.local_camera_stream.get_clone()
        } else {
            voice_ctx
                .remote_video_streams
                .with(|map| map.get(&user_id).cloned())
        }
    });

    let has_video = create_memo(move || {
        if is_me {
            is_camera_on.get() && cam_stream.get_clone().is_some()
        } else {
            is_camera_on.get() || cam_stream.get_clone().is_some()
        }
    });
    let is_pinned = create_memo(move || voice_ctx.pinned_user_id.get() == Some(user_id));

    let avatar_sig = create_signal(avatar);
    let initial_sig = create_signal(initial);
    let username_sig = create_signal(username);

    let on_card_click = move |_| {
        if is_filmstrip {
            voice_ctx.pinned_user_id.set(Some(user_id));
        }
    };

    let on_toggle_pin = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        voice_ctx
            .pinned_user_id
            .set(if is_pinned.get() { None } else { Some(user_id) });
    };

    let on_toggle_fullscreen = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        toggle_element_fullscreen(card_ref);
    };

    view! {
        div(
            r#ref=card_ref,
            class=classes(vec![
                "voice-participant-card".into(),
                ("has-video", has_video.into()).into(),
                ("spotlight", is_spotlight.into()).into(),
                ("filmstrip-card", is_filmstrip.into()).into(),
                ("is-speaking", is_speaking.into()).into(),
                ("is-pinned", is_pinned.into()).into(),
            ]),
            on:click=on_card_click,
        ) {
            // Top-right action overlay (Pin & Fullscreen buttons)
            div(class="tile-actions-overlay") {
                button(
                    class=classes(vec![
                        "tile-action-btn".into(),
                        ("active", is_pinned.into()).into(),
                    ]),
                    title=if is_pinned.get() { "Открепить" } else { "Закрепить в центре" },
                    on:click=on_toggle_pin,
                ) {
                    PinIcon()
                }

                (if has_video.get() {
                    view! {
                        button(
                            class="tile-action-btn",
                            title="Во весь экран",
                            on:click=on_toggle_fullscreen,
                        ) {
                            FullscreenIcon()
                        }
                    }
                } else {
                    view! {}
                })
            }

            // Video Stream or Avatar View (fully reactive dynamic closure)
            (move || {
                if has_video.get() && let Some(stream) = cam_stream.get_clone() {
                    view! {
                        div(class="participant-video-container") {
                            VideoTile(stream=stream, is_mirrored=is_me)
                        }
                        div(class="tile-info-bar") {
                            div(class="tile-info-user") {
                                (if !avatar_sig.get_clone().is_empty() {
                                    let src = avatar_sig.get_clone();
                                    view! { img(src=src, alt="", class="tile-mini-avatar") }
                                } else {
                                    let init = initial_sig.get_clone();
                                    view! { span(class="tile-mini-avatar-placeholder") { (init) } }
                                })
                                span(class="participant-username") { (username_sig.get_clone()) }
                                (if is_me {
                                    view! { span(class="me-tag") { " (Вы)" } }
                                } else {
                                    view! {}
                                })
                            }
                            div(class="tile-info-badges") {
                                (if is_muted.get() {
                                    view! { span(class="tile-badge muted", title="Микрофон выключен") { MicOffIcon() } }
                                } else if is_speaking.get() {
                                    view! { span(class="tile-badge speaking", title="Говорит") { MicIcon() } }
                                } else {
                                    view! {}
                                })

                                (if is_deafened.get() {
                                    view! { span(class="tile-badge deafened", title="Звук выключен") { HeadphonesOffIcon() } }
                                } else {
                                    view! {}
                                })

                                (if is_screen_sharing.get() {
                                    view! { span(class="tile-badge screen", title="Транслирует экран") { ScreenShareIcon() } }
                                } else {
                                    view! {}
                                })
                            }
                        }
                    }
                } else if is_camera_on.get() && !is_me {
                    view! {
                        div(class="participant-video-container connecting") {
                            div(class="voice-connecting-spinner")
                            span(class="video-connecting-text") { "Подключение камеры..." }
                        }
                        div(class="tile-info-bar") {
                            div(class="tile-info-user") {
                                (if !avatar_sig.get_clone().is_empty() {
                                    let src = avatar_sig.get_clone();
                                    view! { img(src=src, alt="", class="tile-mini-avatar") }
                                } else {
                                    let init = initial_sig.get_clone();
                                    view! { span(class="tile-mini-avatar-placeholder") { (init) } }
                                })
                                span(class="participant-username") { (username_sig.get_clone()) }
                            }
                            div(class="tile-info-badges") {
                                span(class="tile-badge video-active", title="Камера подключается") { VideoIcon() }
                            }
                        }
                    }
                } else {
                    view! {
                        div(class="participant-avatar-wrapper") {
                            div(
                                class=classes(vec![
                                    "speaking-pulse-ring".into(),
                                    ("speaking", is_speaking.into()).into(),
                                    ("muted", is_muted.into()).into(),
                                ])
                            ) {}
                            (if !avatar_sig.get_clone().is_empty() {
                                let src = avatar_sig.get_clone();
                                view! { img(src=src, alt="", class="participant-avatar-img") }
                            } else {
                                let init = initial_sig.get_clone();
                                view! {
                                    div(class="participant-avatar-placeholder") {
                                        span { (init) }
                                    }
                                }
                            })
                        }

                        div(class="participant-meta") {
                            div(class="participant-name-row") {
                                span(class="participant-username") { (username_sig.get_clone()) }
                                (if is_me {
                                    view! { span(class="me-tag") { " (Вы)" } }
                                } else {
                                    view! {}
                                })
                            }

                            div(class="participant-status-badges") {
                                (if is_muted.get() {
                                    view! {
                                        span(class="status-pill muted", title="Микрофон выключен") {
                                            MicOffIcon()
                                            span { "Без микрофона" }
                                        }
                                    }
                                } else if is_speaking.get() {
                                    view! {
                                        span(class="status-pill active", title="Микрофон активен") {
                                            MicIcon()
                                            span { "В эфире" }
                                        }
                                    }
                                } else {
                                    view! {
                                        span(class="status-pill idle", title="Микрофон включен") {
                                            MicIcon()
                                            span { "На связи" }
                                        }
                                    }
                                })

                                (if is_deafened.get() {
                                    view! {
                                        span(class="status-pill deafened", title="Звук выключен") {
                                            HeadphonesOffIcon()
                                            span { "Без звука" }
                                        }
                                    }
                                } else {
                                    view! {}
                                })

                                (if is_camera_on.get() {
                                    view! {
                                        span(class="status-pill video-active", title="Камера включена") {
                                            VideoIcon()
                                            span { "Камера" }
                                        }
                                    }
                                } else {
                                    view! {}
                                })

                                (if is_screen_sharing.get() {
                                    view! {
                                        span(class="status-pill screen-share-active", title="Транслирует экран") {
                                            ScreenShareIcon()
                                            span { "Экран" }
                                        }
                                    }
                                } else {
                                    view! {}
                                })
                            }
                        }
                    }
                }
            })
        }
    }
}

#[component(inline_props)]
pub(crate) fn ConnectedRoom<F1, F2, F3, F4, F5>(
    on_toggle_mic: F1,
    on_toggle_deafen: F2,
    on_toggle_camera: F3,
    on_toggle_screen: F4,
    on_disconnect: F5,
) -> View
where
    F1: Fn(web_sys::MouseEvent) + Copy + 'static,
    F2: Fn(web_sys::MouseEvent) + Copy + 'static,
    F3: Fn(web_sys::MouseEvent) + Copy + 'static,
    F4: Fn(web_sys::MouseEvent) + Copy + 'static,
    F5: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    let user_profile = use_context::<UserProfileContext>();
    let voice_ctx = use_context::<VoiceContext>();
    let screen_stage_ref = create_node_ref();

    let is_codec_menu_open = create_signal(false);

    let all_participants = create_memo(move || {
        let list = voice_ctx.participants.get_clone();
        if !list.is_empty() {
            list
        } else {
            vec![VoiceParticipantView {
                user_id: user_profile.user_id.get().unwrap_or_else(|| {
                    gilvave_core::ids::UserId::from(gilvave_core::uuid::Uuid::nil())
                }),
                username: user_profile.username.get_clone(),
                avatar: user_profile.avatar.get_clone(),
                is_muted: user_profile.is_muted.get(),
                is_deafened: user_profile.is_deafened.get(),
                is_speaking: voice_ctx.is_speaking.get(),
                is_camera_on: voice_ctx.is_camera_on.get(),
                is_screen_sharing: voice_ctx.is_screen_sharing.get(),
            }]
        }
    });

    let screen_sharer = create_memo(move || {
        if voice_ctx.is_screen_sharing.get() {
            let name = user_profile.username.get_clone();
            let stream = voice_ctx.local_screen_stream.get_clone();
            return Some((name, stream, true));
        }
        for p in voice_ctx.participants.get_clone().iter() {
            if p.is_screen_sharing {
                let stream = voice_ctx
                    .remote_screen_streams
                    .with(|map| map.get(&p.user_id).cloned());
                return Some((p.username.clone(), stream, false));
            }
        }
        None
    });

    let pinned_participant = create_memo(move || {
        if let Some(pinned_uid) = voice_ctx.pinned_user_id.get() {
            all_participants
                .get_clone()
                .into_iter()
                .find(|p| p.user_id == pinned_uid)
        } else {
            None
        }
    });

    let remaining_participants = create_memo(move || {
        if let Some(pinned_uid) = voice_ctx.pinned_user_id.get() {
            all_participants
                .get_clone()
                .into_iter()
                .filter(|p| p.user_id != pinned_uid)
                .collect()
        } else {
            Vec::new()
        }
    });

    let grid_class = create_memo(move || match all_participants.with(|p| p.len()) {
        0 | 1 => "voice-participants-stage grid-1",
        2 => "voice-participants-stage grid-2",
        3 => "voice-participants-stage grid-3",
        4 => "voice-participants-stage grid-4",
        _ => "voice-participants-stage grid-multi",
    });

    let toggle_codec_menu = move |_| {
        is_codec_menu_open.update(|open| *open = !*open);
    };

    let set_codec = move |codec: VideoCodecPreference| {
        voice_ctx.video_codec_preference.set(codec);
        is_codec_menu_open.set(false);
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let val = match codec {
                VideoCodecPreference::Auto => "AUTO",
                VideoCodecPreference::Vp8 => "VP8",
                VideoCodecPreference::H264 => "H264",
                VideoCodecPreference::Vp9 => "VP9",
                VideoCodecPreference::Av1 => "AV1",
            };
            let _ = storage.set_item("gilvave_video_codec", val);
        }
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::voice::VoiceManager::set_codec_preference(codec).await;
        });
    };

    let on_screen_fullscreen = move |_| {
        toggle_element_fullscreen(screen_stage_ref);
    };

    let on_toggle_focus_mode = move |_| {
        voice_ctx.is_focus_mode.update(|focus| *focus = !*focus);
    };

    view! {
        div(class="voice-connected-room") {
            div(class="voice-stage-viewport") {
                (move || {
                    if let Some((sharer_name, screen_stream, _is_screen_me)) = screen_sharer.get_clone() {
                        view! {
                            div(class="voice-theater-layout") {
                                div(class="voice-screen-share-stage", r#ref=screen_stage_ref) {
                                    div(class="screen-share-header") {
                                        div(class="screen-share-info") {
                                            ScreenShareIcon()
                                            span(class="sharer-name") { (format!("{sharer_name} транслирует экран")) }
                                            span(class="live-badge") { "В ЭФИРЕ" }
                                        }
                                        div(class="screen-share-actions") {
                                            button(
                                                class=classes(vec![
                                                    "theater-action-btn".into(),
                                                    ("active", voice_ctx.is_focus_mode.into()).into(),
                                                ]),
                                                title=if voice_ctx.is_focus_mode.get() { "Показать участников" } else { "Режим кинотеатра" },
                                                on:click=on_toggle_focus_mode,
                                            ) {
                                                span { (if voice_ctx.is_focus_mode.get() { "Сетка" } else { "Кинотеатр" }) }
                                            }
                                            button(
                                                class="theater-action-btn",
                                                title="На весь экран",
                                                on:click=on_screen_fullscreen,
                                            ) {
                                                FullscreenIcon()
                                            }
                                        }
                                    }
                                    div(class="screen-share-video-wrapper") {
                                        (if let Some(stream) = screen_stream {
                                            view! {
                                                VideoTile(stream=stream, is_mirrored=false)
                                            }
                                        } else {
                                            view! {
                                                div(class="screen-share-placeholder") {
                                                    div(class="voice-connecting-spinner")
                                                    span { "Подключение к трансляции..." }
                                                }
                                            }
                                        })
                                    }
                                }

                                (if !voice_ctx.is_focus_mode.get() {
                                    view! {
                                        div(class="voice-filmstrip-strip") {
                                            Indexed(
                                                list=all_participants,
                                                view=|p| view! {
                                                    ParticipantCard(
                                                        participant=p,
                                                        is_spotlight=false,
                                                        is_filmstrip=true,
                                                    )
                                                },
                                            )
                                        }
                                    }
                                } else {
                                    view! {}
                                })
                            }
                        }
                    } else if let Some(pinned_p) = pinned_participant.get_clone() {
                        let has_remaining = create_memo(move || !remaining_participants.with(|r| r.is_empty()));
                        view! {
                            div(class="voice-spotlight-layout") {
                                div(class="voice-spotlight-stage") {
                                    ParticipantCard(
                                        participant=pinned_p,
                                        is_spotlight=true,
                                        is_filmstrip=false,
                                    )
                                }

                                (if has_remaining.get() {
                                    view! {
                                        div(class="voice-filmstrip-strip") {
                                            Indexed(
                                                list=remaining_participants,
                                                view=|p| view! {
                                                    ParticipantCard(
                                                        participant=p,
                                                        is_spotlight=false,
                                                        is_filmstrip=true,
                                                    )
                                                },
                                            )
                                        }
                                    }
                                } else {
                                    view! {}
                                })
                            }
                        }
                    } else {
                        let active_grid_class = grid_class.get();
                        view! {
                            div(class=active_grid_class) {
                                Indexed(
                                    list=all_participants,
                                    view=|p| view! {
                                        ParticipantCard(
                                            participant=p,
                                            is_spotlight=false,
                                            is_filmstrip=false,
                                        )
                                    },
                                )
                            }
                        }
                    }
                })
            }

            // Bottom Controls Floating Dock
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

                // Camera Toggle
                button(
                    class=classes(vec![
                        "voice-control-btn".into(),
                        ("active", voice_ctx.is_camera_on.into()).into(),
                    ]),
                    on:click=on_toggle_camera,
                    title=if voice_ctx.is_camera_on.get() { "Выключить камеру" } else { "Включить камеру" },
                ) {
                    (if voice_ctx.is_camera_on.get() {
                        view! { VideoIcon() }
                    } else {
                        view! { VideoOffIcon() }
                    })
                    span {
                        (if voice_ctx.is_camera_on.get() {
                            "Камера вкл."
                        } else {
                            "Камера"
                        })
                    }
                }

                // Screen Share Toggle
                button(
                    class=classes(vec![
                        "voice-control-btn".into(),
                        ("active", voice_ctx.is_screen_sharing.into()).into(),
                    ]),
                    on:click=on_toggle_screen,
                    title=if voice_ctx.is_screen_sharing.get() { "Остановить демонстрацию" } else { "Демонстрация экрана" },
                ) {
                    (if voice_ctx.is_screen_sharing.get() {
                        view! { ScreenShareIcon() }
                    } else {
                        view! { ScreenShareOffIcon() }
                    })
                    span {
                        (if voice_ctx.is_screen_sharing.get() {
                            "Экран вкл."
                        } else {
                            "Экран"
                        })
                    }
                }

                // Dynamic Codec Selector Popover
                div(class="voice-codec-selector-wrapper") {
                    button(
                        class=classes(vec![
                            "voice-control-btn codec-btn".into(),
                            ("active", is_codec_menu_open.into()).into(),
                        ]),
                        on:click=toggle_codec_menu,
                        title="Выбрать видеокодек для трансляций",
                    ) {
                        GearIcon()
                        span { (voice_ctx.video_codec_preference.get().short_name()) }
                    }

                    (if is_codec_menu_open.get() {
                        view! {
                            div(class="voice-codec-popover") {
                                div(class="codec-popover-title") { "Видеокодек WebRTC" }
                                div(class="codec-popover-desc") { "Кодек применяется при следующей публикации" }

                                button(
                                    class=classes(vec![
                                        "codec-option".into(),
                                        ("selected", (voice_ctx.video_codec_preference.get() == VideoCodecPreference::Auto).into()).into(),
                                    ]),
                                    on:click=move |_| set_codec(VideoCodecPreference::Auto),
                                ) {
                                    div(class="codec-option-header") {
                                        span(class="codec-name") { "Авто (VP8)" }
                                        span(class="codec-tag recommended") { "Рекомендуется" }
                                    }
                                    span(class="codec-details") { "Автоматический выбор оптимального кодека" }
                                }

                                button(
                                    class=classes(vec![
                                        "codec-option".into(),
                                        ("selected", (voice_ctx.video_codec_preference.get() == VideoCodecPreference::Vp8).into()).into(),
                                    ]),
                                    on:click=move |_| set_codec(VideoCodecPreference::Vp8),
                                ) {
                                    div(class="codec-option-header") {
                                        span(class="codec-name") { "VP8" }
                                        span(class="codec-tag") { "Базовый" }
                                    }
                                    span(class="codec-details") { "Высокая совместимость со всеми браузерами" }
                                }

                                button(
                                    class=classes(vec![
                                        "codec-option".into(),
                                        ("selected", (voice_ctx.video_codec_preference.get() == VideoCodecPreference::H264).into()).into(),
                                    ]),
                                    on:click=move |_| set_codec(VideoCodecPreference::H264),
                                ) {
                                    div(class="codec-option-header") {
                                        span(class="codec-name") { "H.264" }
                                        span(class="codec-tag hardware") { "GPU Ускорение" }
                                    }
                                    span(class="codec-details") { "Минимальная нагрузка на процессор" }
                                }

                                button(
                                    class=classes(vec![
                                        "codec-option".into(),
                                        ("selected", (voice_ctx.video_codec_preference.get() == VideoCodecPreference::Vp9).into()).into(),
                                    ]),
                                    on:click=move |_| set_codec(VideoCodecPreference::Vp9),
                                ) {
                                    div(class="codec-option-header") {
                                        span(class="codec-name") { "VP9" }
                                        span(class="codec-tag hd") { "HD Четкость" }
                                    }
                                    span(class="codec-details") { "Улучшенное сжатие для высокого разрешения" }
                                }

                                button(
                                    class=classes(vec![
                                        "codec-option".into(),
                                        ("selected", (voice_ctx.video_codec_preference.get() == VideoCodecPreference::Av1).into()).into(),
                                    ]),
                                    on:click=move |_| set_codec(VideoCodecPreference::Av1),
                                ) {
                                    div(class="codec-option-header") {
                                        span(class="codec-name") { "AV1" }
                                        span(class="codec-tag modern") { "Новейший" }
                                    }
                                    span(class="codec-details") { "Максимальная эффективность сжатия нового поколения" }
                                }
                            }
                        }
                    } else {
                        view! {}
                    })
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
}
