use gilvave_core::{
    dto::server::{ServerCreateInfo, ServerSmallPart},
    ids::ServerId,
};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::common::{ChannelContext, ServerContext},
    http::api::Api,
};

pub(super) fn select_server(server_id: ServerId) {
    let context = use_context::<ServerContext>();
    let ch_ctx = use_context::<ChannelContext>();
    if context
        .current
        .with(|s| s.as_ref().is_some_and(|srv| srv.id == server_id))
    {
        ch_ctx.current.set(None);
        return;
    }

    ch_ctx.current.set(None);
    ch_ctx.messages.set(vec![]);

    if let Some(small) = context.list.with(|l| l.iter().find(|s| s.id == server_id).cloned()) {
        let epoch = time::OffsetDateTime::from_unix_timestamp(0).unwrap();
        let current_srv = gilvave_core::dto::server::Server {
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

    spawn_local_scoped(async move {
        if let Ok(server) = Api::get_server_by_id(server_id).await {
            context.current.set(Some(server));
        }
    });
}

pub(super) fn create_server(
    server_list: Signal<Vec<ServerSmallPart>>,
    server_info: ServerCreateInfo,
) {
    spawn_local_scoped(async move {
        if let Ok(server) = Api::create_server(server_info).await {
            server_list.update(|list| {
                list.push(ServerSmallPart {
                    id: server.id,
                    name: server.name,
                    icon_url: server.icon_url,
                })
            });
        }
    });
}
