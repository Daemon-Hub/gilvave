//! Top header server information and controls.

use sycamore::prelude::*;
use gilvave_core::dto::servers::Server;
use crate::components::{
    common::UiModalContext,
    ui::icons::{GearIcon, HomeIcon},
};

pub(crate) fn header_server_view(
    server_signal: Signal<Option<Server>>,
    modal_context: UiModalContext,
) -> View {
    let header_data = server_signal.with(|server_opt| {
        server_opt.as_ref().map(|server| {
            let cover = (!server.cover.is_empty()).then(|| server.cover.clone());
            let icon = (!server.icon_url.is_empty()).then(|| server.icon_url.clone());
            let initial = server.icon_url.is_empty().then(|| {
                server
                    .name
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string()
            });
            let members_str = format!("{} участников", server.members_count);
            let s_name = server.name.clone();
            (cover, icon, initial, s_name, members_str)
        })
    });

    if let Some((cover, icon, initial, s_name, members_str)) = header_data {
        let cover_view = if let Some(c) = cover {
            view! {
                div(class="header-server-cover") {
                    img(src=c, alt="")
                }
            }
        } else {
            view! {}
        };

        let icon_view = if let Some(i) = icon {
            view! { img(src=i, alt="") }
        } else {
            let init_str = initial.unwrap_or_default();
            view! { span { (init_str) } }
        };

        view! {
            div(class="header-server-info") {
                (cover_view)
                div(class="header-server-icon") {
                    (icon_view)
                }
                div(class="header-server-text") {
                    span(class="header-server-name") { (s_name) }
                    span(class="header-server-members") { (members_str) }
                }
                button(
                    class="header-settings-btn",
                    on:click=move |_| modal_context.is_server_settings_open.set(true),
                    title="Настройки сервера",
                ) {
                    GearIcon()
                }
            }
        }
    } else {
        view! {
            div(class="header-server-info") {
                div(class="header-server-icon home") {
                    HomeIcon()
                }
                div(class="header-server-text") {
                    span(class="header-server-name") { "Личные сообщения" }
                }
            }
        }
    }
}
