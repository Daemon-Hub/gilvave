//! Pre-connect stage with radar pulse rings, audio specifications, and connect action.

use crate::components::{
    common::{UserProfileContext, VoiceConnectionStatus, VoiceContext},
    ui::icons::{HeadphonesIcon, MicIcon, MicOffIcon, PhoneIcon, Volume2Icon},
};
use sycamore::prelude::*;

#[component(inline_props)]
pub(crate) fn PreConnectCard<F>(channel_name: ReadSignal<String>, on_connect: F) -> View
where
    F: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    let user_profile = use_context::<UserProfileContext>();
    let voice_ctx = use_context::<VoiceContext>();

    let is_connecting =
        create_memo(move || voice_ctx.connection_status.get() == VoiceConnectionStatus::Connecting);

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

            button(
                class="voice-connect-action-btn",
                disabled=is_connecting,
                on:click=on_connect,
            ) {
                PhoneIcon()
                span {
                    (if is_connecting.get() {
                        "Подключение..."
                    } else {
                        "Подключиться к голосовому каналу"
                    })
                }
            }
        }
    }
}
