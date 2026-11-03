//! Channel chat header and welcome banner components.

use sycamore::prelude::*;

#[component(inline_props)]
pub fn ChannelHeader<F>(channel_name: ReadSignal<String>, on_back: F) -> View
where
    F: Fn(web_sys::MouseEvent) + 'static,
{
    view! {
        div(class="chat-header") {
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
            div(class="chat-header-title") {
                span(class="channel-hash") { "#" }
                span(class="channel-name") { (channel_name) }
            }
        }
    }
}

#[component(inline_props)]
pub fn WelcomeMessage(channel_name: ReadSignal<String>) -> View {
    view! {
        div(class="welcome-message") {
            p { (format!("Добро пожаловать на канал {channel_name}!")) }
        }
    }
}
