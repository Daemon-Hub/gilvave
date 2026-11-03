//! Voice channel header bar with channel name, connected badge, and server name.

use sycamore::prelude::*;
use crate::components::{
    common::classes,
    ui::icons::Volume2Icon,
};

#[component(inline_props)]
pub(crate) fn VoiceHeader<F>(
    channel_name: ReadSignal<String>,
    server_name: ReadSignal<String>,
    is_connected: Signal<bool>,
    on_back: F,
) -> View
where
    F: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    view! {
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
    }
}
