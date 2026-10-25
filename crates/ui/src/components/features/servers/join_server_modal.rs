use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::JsCast;

use crate::{
    components::{
        common::CreateServerContext,
        ui::{
            ShowMoreBanner,
            icons::{ArrowLeftIcon, CloseSmallIcon},
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
    let is_loading = create_signal(false);
    let body_ref = create_node_ref();

    let is_banner_visible = create_signal(false);
    create_effect(move || {
        let visible =
            context.has_more_public_servers.get() && (is_at_bottom.get() || is_loading.get());
        is_banner_visible.set(visible);
    });

    let on_close_click = {
        let on_c = on_close;
        move |e: web_sys::MouseEvent| on_c(e)
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
            on:click=on_close_click,
        ) {
            div(class="server-modal join-modal", on:click=move |event: web_sys::MouseEvent| event.stop_propagation()) {
                div(class="server-modal-header") {
                    span { "Присоединиться к серверу" }
                    button(class="modal-close-icon-btn", on:click=move |_| {
                        let ctx = use_context::<CreateServerContext>();
                        ctx.is_modal_open.set(false);
                    }, title="Закрыть") { CloseSmallIcon() }
                    div(class="join-modal-search") {
                        input(r#type="text", placeholder="Поиск серверов...")
                    }
                }
                div(
                    class="join-modal-body",
                    r#ref=body_ref,
                    on:scroll=on_scroll,
                ) {
                    div(class="join-modal-grid") {
                        Indexed(
                            list=context.public_servers,
                            view=move |server| server_card(server, context.expanded_id),
                        )
                    }
                }

                ShowMoreBanner(
                    is_visible=is_banner_visible,
                    is_loading=is_loading,
                    on_click=handle_load_more,
                )

                div(class="join-modal-footer") {
                    button(class="back-btn", on:click=on_back) {
                        ArrowLeftIcon()
                        span { "Назад" }
                    }
                }
            }
        }
    }
}
