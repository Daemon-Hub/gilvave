use sycamore::prelude::*;

use crate::components::{
    common::CreateServerContext,
    ui::icons::{ArrowLeftIcon, CloseSmallIcon},
};

use super::server_card::server_card;

#[component(inline_props)]
pub(super) fn JoinServerModal(
    on_back: impl Fn(web_sys::MouseEvent) + 'static,
    on_close: impl Fn(web_sys::MouseEvent) + 'static,
) -> View {
    let context = use_context::<CreateServerContext>();

    let on_close_click = {
        let on_c = on_close;
        move |e: web_sys::MouseEvent| on_c(e)
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
                div(class="join-modal-body") {
                    div(class="join-modal-grid") {
                        Indexed(
                            list=context.public_servers,
                            view=move |server| server_card(server, context.expanded_id),
                        )
                    }
                }
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
