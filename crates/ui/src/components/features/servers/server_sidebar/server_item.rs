//! Individual server icon with active pill indicator and click selection.

use gilvave_core::dto::servers::ServerSmallPart;
use sycamore::prelude::*;

use crate::components::{
    common::{CreateServerContext, ServerContext, UiModalContext, classes},
    features::servers::server_actions::select_server,
    ui::icons::ServerIcon,
};

#[component(inline_props)]
pub fn ServerIconItem(server: ServerSmallPart) -> View {
    let server_context = use_context::<ServerContext>();
    let id = server.id;
    let tip = server.name.clone();
    let name = create_signal(server.name);
    let icon_url = create_signal(server.icon_url);

    let is_active_memo = create_memo(move || {
        server_context.current.with(|c| c.as_ref().map(|s| s.id) == Some(id))
    });
    let is_active_dyn: MaybeDyn<bool> = is_active_memo.into();

    let pill_class = classes(vec![
        "server-icon-pill".into(),
        ("active", is_active_dyn.clone()).into(),
    ]);

    let s_id_str = id.to_string();
    let s_name_str = tip.clone();

    view! {
        div(
            class="server-icon-wrapper",
            data-server-id=s_id_str,
            data-server-name=s_name_str,
        ) {
            div(class=pill_class)
            ServerIcon(
                server_name=name,
                icon_url=icon_url,
                is_active=is_active_dyn,
                data-tip=tip,
                on:click=move |_| {
                    let c_ctx = use_context::<CreateServerContext>();
                    c_ctx.is_modal_open.set(false);
                    if let Some(m_ctx) = try_use_context::<UiModalContext>() {
                        m_ctx.selected_dm_name.set(None);
                    }
                    select_server(id);
                },
            )
        }
    }
}
