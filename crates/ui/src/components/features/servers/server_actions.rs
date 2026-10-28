use gilvave_core::{
    dto::servers::{Server, ServerCreateInfo, ServerSmallPart},
    ids::ServerId,
};
use sycamore::prelude::*;

use crate::{
    components::common::{ChannelContext, ServerContext},
    gateway::service::WsService,
    http::api::Api,
};

pub fn select_server(server_id: ServerId) {
    let context = use_context::<ServerContext>();
    let ch_ctx = use_context::<ChannelContext>();
    if context
        .current
        .with(|s| s.as_ref().is_some_and(|srv| srv.id == server_id))
    {
        ch_ctx.current.set(None);
        return;
    }

    let active_channel_id = ch_ctx.current.with(|c| c.as_ref().map(|ch| ch.id));
    ch_ctx.current.set(None);
    ch_ctx.messages.set(vec![]);
    if let Some(channel_id) = active_channel_id {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = WsService::left_channel(channel_id).await;
        });
    }

    if let Some(small) = context
        .list
        .with(|l| l.iter().find(|s| s.id == server_id).cloned())
    {
        let epoch = time::OffsetDateTime::from_unix_timestamp(0).unwrap();
        let current_srv = Server {
            id: small.id,
            owner_id: gilvave_core::ids::UserId::default(),
            name: small.name,
            description: String::new(),
            icon_url: small.icon_url,
            cover: String::new(),
            is_public: true,
            members_count: 1,
            created_at: epoch,
        };
        context.current.set(Some(current_srv));
    }

    context.members.set(vec![]);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(server) = Api::get_server_by_id(server_id).await {
            context.current.set(Some(server));
        }
    });
}

pub(super) fn create_server(
    server_list: Signal<Vec<ServerSmallPart>>,
    server_info: ServerCreateInfo,
) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(server) = Api::create_server(server_info).await {
            #[cfg(target_arch = "wasm32")]
            super::server_overview_modal::add_owned_server_id(server.id);

            server_list.update(|list| {
                list.push(ServerSmallPart {
                    id: server.id,
                    name: server.name,
                    icon_url: server.icon_url,
                    is_owner: true,
                })
            });
        }
    });
}
