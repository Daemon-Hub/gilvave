//! Connected voice channel room stage with participant cards and floating dock controls.

use sycamore::prelude::*;
use crate::components::{
    common::{classes, UserProfileContext},
    ui::icons::{
        HeadphonesIcon, HeadphonesOffIcon, MicIcon, MicOffIcon, PhoneOffIcon,
    },
};

#[component(inline_props)]
pub(crate) fn ConnectedRoom<F1, F2, F3>(
    on_toggle_mic: F1,
    on_toggle_deafen: F2,
    on_disconnect: F3,
) -> View
where
    F1: Fn(web_sys::MouseEvent) + Copy + 'static,
    F2: Fn(web_sys::MouseEvent) + Copy + 'static,
    F3: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    let user_profile = use_context::<UserProfileContext>();

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
}
