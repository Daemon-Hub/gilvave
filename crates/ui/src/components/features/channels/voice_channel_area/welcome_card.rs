//! Pre-connect stage with radar pulse rings, audio specifications, and connect action.

use sycamore::prelude::*;
use crate::components::{
    common::UserProfileContext,
    ui::icons::{HeadphonesIcon, MicIcon, MicOffIcon, PhoneIcon, Volume2Icon},
};

#[component(inline_props)]
pub(crate) fn PreConnectCard<F>(channel_name: ReadSignal<String>, on_connect: F) -> View
where
    F: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    let user_profile = use_context::<UserProfileContext>();

    view! {
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
}
