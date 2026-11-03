//! Server sidebar: list of joined servers, active indicators, and add-server action.

pub mod create_modal;
pub mod server_item;

pub use create_modal::open_join_modal;

use sycamore::prelude::*;

use self::create_modal::AddServerFlowModal;
use self::server_item::ServerIconItem;
use crate::components::{
    common::{CreateServerContext, CreateServerOrigin, ModalView, ServerContext},
    ui::icons::ServerIcon,
};

#[component(inline_props)]
pub fn ServerSidebar() -> View {
    let server_context = use_context::<ServerContext>();

    let on_plus_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        context.from_dashboard.set(false);
        context.origin.set(CreateServerOrigin::Sidebar);
        context.modal_view.set(ModalView::Home);
        context.is_modal_open.set(true);
    };

    view! {
        Indexed(
            list=server_context.list,
            view=|server| {
                view! {
                    ServerIconItem(server=server)
                }
            },
        )
        ServerIcon(
            server_name=create_signal("+".to_string()),
            icon_url=create_signal(String::default()),
            data-tip="Добавить сервер",
            on:click=on_plus_click,
        )

        AddServerFlowModal()
    }
}
