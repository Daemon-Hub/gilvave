use gilvave_core::{
    dto::servers::{Server, ServerSmallPart},
    ids::ServerId,
};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::common::{ChannelContext, CreateServerContext, ServerContext, classes},
    http::api::Api,
};

pub(super) fn card_icon(icon_url: String, first_char: String) -> View {
    if icon_url.is_empty() {
        view! { span { (first_char) } }
    } else {
        view! { img(src=icon_url) }
    }
}

pub(super) fn server_card(server: Server, expanded_id: Signal<Option<ServerId>>) -> View {
    let id = server.id;
    let name = server.name.clone();
    let name_sm = server.name.clone();
    let first_char = server
        .name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let desc_short = {
        let truncated: String = server.description.chars().take(60).collect();
        if truncated.len() < server.description.len() {
            format!("{truncated}...")
        } else {
            truncated
        }
    };

    let toggle_expand = move |_| {
        expanded_id.set((expanded_id.get() != Some(id)).then_some(id));
    };

    let server_c = server.clone();
    let join_server = move |_| {
        let server_cc = server_c.clone();
        spawn_local_scoped(async move {
            if Api::join_public_server(server_cc.id).await.is_ok() {
                let context = use_context::<ServerContext>();
                context.list.update(|list| {
                    list.push(ServerSmallPart {
                        id: server_cc.id,
                        name: server_cc.name.clone(),
                        icon_url: server_cc.icon_url.clone(),
                        is_owner: false,
                    })
                });
                context.current.set(Some(server_cc));
                let ch_context = use_context::<ChannelContext>();
                ch_context.current.set(None);
                ch_context.messages.set(vec![]);
                let context = use_context::<CreateServerContext>();
                context.is_modal_open.set(false);
            }
        });
    };

    let cover_url = server.cover.clone();
    let center_icon = card_icon(server.icon_url.clone(), first_char.clone());
    let bottom_icon = card_icon(server.icon_url, first_char);

    view! {
        div(
            class=classes(vec![
                "server-browser-card".into(),
                ("expanded", { expanded_id.get() == Some(id) }.into()).into(),
            ]),
            on:click=toggle_expand,
        ) {
            div(class="card-cover") {
                (if !cover_url.is_empty() {
                    let cover = cover_url.clone();
                    view! { img(src=cover, alt="") }
                } else {
                    view! {}
                })
            }
            div(
                class=classes(vec![
                    "card-collapsed-overlay".into(),
                    ("hidden", { expanded_id.get() == Some(id) }.into()).into(),
                ]),
            ) {
                div(class="card-icon-bottom") { (bottom_icon) }
                div(class="card-collapsed-text") {
                    div(class="card-server-name-sm") { (name_sm) }
                    div(class="card-desc-short") { (desc_short) }
                }
            }
            div(
                class=classes(vec![
                    "card-expanded-body".into(),
                    ("hidden", { expanded_id.get() != Some(id) }.into()).into(),
                ]),
            ) {
                div(class="card-icon-centered") { (center_icon) }
                div(class="card-server-name") { (name) }
                div(class="card-description") { (server.description) }
                div(class="card-members") {
                    svg(
                        xmlns="http://www.w3.org/2000/svg", width="14", height="14",
                        viewBox="0 0 24 24", fill="none", stroke="currentColor",
                        stroke-width="2", stroke-linecap="round", stroke-linejoin="round",
                    ) {
                        path(d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2")
                        circle(cx="9", cy="7", r="4")
                        path(d="M23 21v-2a4 4 0 0 0-3-3.87")
                        path(d="M16 3.13a4 4 0 0 1 0 7.75")
                    }
                    span { (format!("{} участников", server.members_count)) }
                }
                div(class="card-join-row") {
                    button(class="card-join-btn", on:click=join_server) { "Присоединиться" }
                }
            }
        }
    }
}
