use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::JsCast;

use crate::{
    components::{
        common::CreateServerContext,
        ui::{
            ShowMoreBanner,
            icons::{ArrowLeftIcon, CloseSmallIcon, GlobeIcon},
        },
    },
    http::api::Api,
};

use super::server_card::server_card;

#[component(inline_props)]
pub(super) fn JoinServerModal(
    on_back: impl Fn(web_sys::MouseEvent) + 'static,
    on_close: impl Fn(web_sys::MouseEvent) + 'static,
) -> View {
    let context = use_context::<CreateServerContext>();
    let is_at_bottom = create_signal(false);
    let is_loading = create_signal(context.public_servers.with(|s| s.is_empty()));
    let search_query = create_signal(String::new());
    let body_ref = create_node_ref();

    // Fetch initial page 1 on mount if list is empty
    if context.public_servers.with(|s| s.is_empty()) {
        spawn_local_scoped(async move {
            match Api::get_public_servers(1).await {
                Ok((servers, has_more)) => {
                    context.public_servers.set(servers);
                    context.has_more_public_servers.set(has_more);
                }
                Err(_) => {
                    context.public_servers.set(vec![]);
                    context.has_more_public_servers.set(false);
                }
            }
            is_loading.set(false);
        });
    }

    let filtered_servers = create_memo(move || {
        let q = search_query.get_clone().trim().to_lowercase();
        let servers = context.public_servers.get_clone();
        if q.is_empty() {
            servers
        } else {
            servers
                .into_iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q)
                })
                .collect()
        }
    });

    let is_banner_visible = create_signal(false);
    create_effect(move || {
        let visible =
            context.has_more_public_servers.get() && (is_at_bottom.get() || is_loading.get());
        is_banner_visible.set(visible);
    });

    let on_close = std::rc::Rc::new(on_close);
    let on_back = std::rc::Rc::new(on_back);

    let on_overlay_close = {
        let on_c = std::rc::Rc::clone(&on_close);
        move |e: web_sys::MouseEvent| on_c(e)
    };

    let on_icon_close = {
        let on_c = std::rc::Rc::clone(&on_close);
        move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            on_c(e);
        }
    };

    let on_back_click = {
        let on_b = std::rc::Rc::clone(&on_back);
        move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            on_b(e);
        }
    };

    let check_bottom = move || {
        if let Some(node) = body_ref.try_get() {
            let el = node.unchecked_into::<web_sys::HtmlElement>();
            let top = el.scroll_top();
            let sh = el.scroll_height();
            let ch = el.client_height();
            let distance = sh - ch - top;
            is_at_bottom.set(distance <= 64);
        }
    };

    let on_scroll = move |_| {
        check_bottom();
    };

    let handle_load_more = move |_| {
        if is_loading.get() || !context.has_more_public_servers.get() {
            return;
        }
        is_loading.set(true);
        let next_page = context.public_servers_page.get() + 1;
        spawn_local_scoped(async move {
            if let Ok((new_servers, has_more)) = Api::get_public_servers(next_page).await {
                let mut current = context.public_servers.get_clone();
                current.extend(new_servers);
                context.public_servers.set(current);
                context.public_servers_page.set(next_page);
                context.has_more_public_servers.set(has_more);
                is_at_bottom.set(false);
            }
            is_loading.set(false);
        });
    };

    view! {
        div(
            class="server-modal-overlay large",
            on:click=on_overlay_close,
        ) {
            div(class="server-modal join-modal", on:click=move |event: web_sys::MouseEvent| event.stop_propagation()) {
                div(class="server-modal-header") {
                    span { "Присоединиться к серверу" }
                    button(
                        class="modal-close-icon-btn",
                        on:click=on_icon_close,
                        title="Закрыть",
                    ) { CloseSmallIcon() }
                    div(class="join-modal-search") {
                        input(
                            r#type="text",
                            placeholder="Поиск серверов...",
                            bind:value=search_query,
                        )
                    }
                }
                div(
                    class="join-modal-body",
                    r#ref=body_ref,
                    on:scroll=on_scroll,
                ) {
                    (if filtered_servers.with(|l| l.is_empty()) {
                        if is_loading.get() {
                            view! {
                                div(class="join-modal-empty") {
                                    span { "Загрузка серверов..." }
                                }
                            }
                        } else {
                            view! {
                                div(class="join-modal-empty") {
                                    GlobeIcon()
                                    span { "Серверы не найдены" }
                                }
                            }
                        }
                    } else {
                        view! {
                            div(class="join-modal-grid") {
                                Indexed(
                                    list=filtered_servers,
                                    view=move |server| server_card(server, context.expanded_id),
                                )
                            }
                        }
                    })
                }

                ShowMoreBanner(
                    is_visible=is_banner_visible,
                    is_loading=is_loading,
                    on_click=handle_load_more,
                )

                div(class="join-modal-footer") {
                    button(class="back-btn", on:click=on_back_click) {
                        ArrowLeftIcon()
                        span { "Назад" }
                    }
                }
            }
        }
    }
}
